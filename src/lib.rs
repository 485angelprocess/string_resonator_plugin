mod delay;
mod filter;
mod model;
mod lfo;

use lfo::LFO;
use model::Model;
use filter::{Filter, SimperFilter};
use std::sync::Arc;
use nih_plug::prelude::*;

use nih_plug_vizia::ViziaState;

mod editor;

fn note_to_freq(n: f32, base: f32) -> f32{
    base*f32::powf(2.0, n / 12.0)
}

fn freq_to_note(f: f64, base: f64) -> f64{
    f64::ln(f / base) / f64::ln(2.0)
}

const NUM_STRINGS: usize = 4;

struct StringModel{
    params: Arc<StringParams>,
    model: [Model<f32, SimperFilter<f32>>; NUM_STRINGS],
    lfo: lfo::LFO
}

#[derive(Params)]
struct StringParams{
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    
    #[id = "dry"]
    pub dry: FloatParam,
    
    #[id = "wet"]
    pub wet: FloatParam,
    
    #[id = "note"]
    pub base: IntParam,

    #[id = "lfo rate"]
    pub lfo_rate: FloatParam,
    
    #[id = "lfo depth"]
    pub lfo_depth: FloatParam,
        
    #[nested(array, group = "String Parameters")]
    pub element_params: [ElementParams; NUM_STRINGS]

}

#[derive(Params)]
struct ElementParams{
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "offset"]
    pub offset: IntParam,
    #[id = "Cutoff"]
    pub cutoff: FloatParam,
    #[id = "Q"]
    pub q: FloatParam
}

fn db_param(name: &str, min: f32, max: f32) -> FloatParam{
    FloatParam::new(
        name,
        util::db_to_gain(0.0),
        FloatRange::Skewed { 
            min: util::db_to_gain(min), 
            max: util::db_to_gain(max), 
            factor: FloatRange::gain_skew_factor(min, max) }
    )
    .with_smoother(SmoothingStyle::Logarithmic(50.0))
    .with_unit(" dB")
    .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
    .with_string_to_value(formatters::s2v_f32_gain_to_db())
}

impl Default for ElementParams{
    fn default() -> Self {
        Self{
            gain: db_param("gain", -30.0, 0.0),
            offset: IntParam::new("Note offset", 0, 
                IntRange::Linear { min: 0, max: 36 }),
            cutoff: FloatParam::new("Cutoff", 440.0,
                    FloatRange::Linear { min: 10.0, max: 22000.0 }),
            q: FloatParam::new("Q", 0.771, FloatRange::Linear { min: 0.5, max: 1.0 })
        }
    }
}

impl Default for StringParams{
    fn default() -> Self {
        Self{
            editor_state: editor::default_state(),
            dry: db_param("dry", -60.0,20.0),
            wet: db_param("wet", -60.0,20.0),
            base: IntParam::new("Note base", 0, IntRange::Linear { min: -44, max: 44 }),
            lfo_rate: FloatParam::new("Lfo Rate", 1.0, FloatRange::Linear { min: 0.1, max: 30.0 }),
            lfo_depth: FloatParam::new("Lfo Depth", 0.0, FloatRange::Linear { min: 0.0, max: 5.0 }),
            element_params: Default::default()
        }
    }
}

impl Default for StringModel{
    fn default() -> Self {
        Self{
            params: Arc::new(StringParams::default()),
            model: Default::default(),
            lfo: LFO::default()
        }
    }
}

impl Plugin for StringModel{
    const NAME: &'static str = "String Resonant Model";
    const VENDOR: &'static str = "Angel Process";
    const URL: &'static str = "485angelprocess.github.com";
    const EMAIL: &'static str = "dontemail@zoinks.band";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            // Individual ports and the layout as a whole can be named here. By default these names
            // are generated as needed. This layout will be called 'Stereo', while the other one is
            // given the name 'Mono' based no the number of input and output channels.
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];
    
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    
    type SysExMessage = ();
    type BackgroundTask = ();
    
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }
    
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples(){
            
            let wet_gain = self.params.wet.smoothed.next();
            let dry_gain = self.params.dry.smoothed.next();
            
            self.lfo.amount = self.params.lfo_depth.smoothed.next();
            self.lfo.set_freq(self.params.lfo_rate.smoothed.next());
            
            let mut elem_gain = [0.0; NUM_STRINGS];
            
            for i in 0..self.model.len(){
                elem_gain[i] = wet_gain * self.params.element_params[i].gain.smoothed.next();
                self.model[i].filter.set_cutoff(self.params.element_params[i].cutoff.smoothed.next());
                self.model[i].filter.set_q(self.params.element_params[i].q.smoothed.next());
            }
            
            // Base note of delay
            let base_note = self.params.base.smoothed.next();
            
            let mut note = [0.0; NUM_STRINGS];
            
            // Each element
            for i in 0..self.model.len(){
                // TODO add fine tuning
                note[i] = (base_note + self.params.element_params[i].offset.smoothed.next()) as f32;
            }
            
            // Get each contribution
            for sample in channel_samples{
                let dry_value = *sample;
                *sample = dry_gain * dry_value;
                let lfo_value = self.lfo.next();
                
                for i in 0..self.model.len(){
                    self.model[i].delay.set_frequency(note_to_freq(note[i] + lfo_value, 440.0), 441000.0);
                    *sample += (elem_gain[i]) * self.model[i].process(dry_value);
                }
                
                // Clip
                if *sample < -1.0{
                    *sample = 0.0;
                }
                if *sample > 1.0{
                    *sample = 1.0;
                }
            }
        }
        ProcessStatus::Normal
    }
    
    fn deactivate(&mut self) {}
}

impl Vst3Plugin for StringModel {
    const VST3_CLASS_ID: [u8; 16] = *b"StringPlugObject";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

impl ClapPlugin for StringModel {
    const CLAP_ID: &'static str = "com.angel-process.string";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("String resonator");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

nih_export_clap!(StringModel);
nih_export_vst3!(StringModel);