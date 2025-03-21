/* Filter implementations */
use simper_filter::{Svf, SvfCoefficients, FilterType};
use num_traits::{Float, FromPrimitive};

pub trait Filter<F: Float + Default>{
    fn tick(&mut self, _input: F) -> F {
        F::default() 
    } 
    fn set_cutoff(&mut self, _cutoff: F){
        ()
    }
    fn set_q(&mut self, _q: F){
        ()
    }
}

#[derive(Default, Clone)]
pub struct FilterSetting<F: Float + Copy>{
    pub filter_type: FilterType,
    pub sample_rate: F,
    pub cutoff: F,
    pub q: F,
    pub gain: F
}

impl<F: Float + Default> FilterSetting<F>{
    pub fn to_svf_coeff(&self) -> SvfCoefficients<F>{
        let mut coeffs = SvfCoefficients::default();
        let _result = coeffs.set(self.filter_type.clone(), self.sample_rate, self.cutoff, self.q, self.gain);
        coeffs
    }
}

pub struct CombFilter<F: Float>{
    x: [F; 4],
    a: F,
    b: F
}

impl<F: Float + Default  + FromPrimitive> Default for CombFilter<F>{
    fn default() -> Self {
        Self{
            x: Default::default(),
            a: F::from_f64(0.5).unwrap(),
            b: F::from_f64(0.5).unwrap()
        }
    }
}

impl <F: Float + Default + FromPrimitive> Filter<F> for CombFilter<F>{
    fn tick(&mut self, input: F) -> F {
        self.x[0] = (self.b * self.x[0]) + (self.a * input);
        self.x[0]
    }
    fn set_cutoff(&mut self, cutoff: F) {
        self.b = F::powf(F::from_f64(2.71828).unwrap(), F::from_f64(-2.0 * 3.14159).unwrap() * cutoff);
        self.a = F::from_f64(1.0).unwrap() - self.b
    }
}

pub struct SimperFilter<F: Float>{
    pub filter: Svf<F>,
    setting: FilterSetting<F>
}

impl<F: Float + Default + FromPrimitive>Default for SimperFilter<F>{
    fn default() -> Self {
        SimperFilter::new(FilterType::Bandpass, 
            F::from_f64(444100.0).unwrap(),
            F::from_f64(1500.0).unwrap(),
            F::from_f64(0.771).unwrap(),
            F::from_f64(0.8).unwrap())
    } 
}

impl<F: Float + Default> Clone for SimperFilter<F>{
    fn clone(&self) -> Self {
        let mut svf = Svf::default();
        svf.set_coeffs(self.setting.to_svf_coeff());
        Self{
            filter: svf,
            setting: self.setting.clone()
        }
    }
}

impl<F: Float + Default> Filter<F> for SimperFilter<F>{
    fn tick(&mut self, input: F) -> F {
        self.filter.tick(input)   
    }
    fn set_cutoff(&mut self, cutoff: F) {
        self.setting.cutoff = cutoff;
        self.set(); 
    }
    fn set_q(&mut self, q: F) {
        self.setting.q = q;
        self.set();
    }
}

impl<F: Float + Default> SimperFilter<F>{
    pub fn new(filter_type: FilterType,
        sample_rate: F,
        cutoff: F,
        q: F,
        gain: F) -> Self{
            let mut svf = Svf::default();
            
            let mut setting = FilterSetting::default();
            
            setting.filter_type = filter_type;
            setting.sample_rate = sample_rate;
            setting.cutoff = cutoff;
            setting.q = q;
            setting.gain = gain;
            
            svf.set_coeffs(setting.to_svf_coeff());
            
            Self{
                filter: svf,
                setting: setting
            }
    }
    
    fn set(&mut self){
        self.filter.set_coeffs(self.setting.to_svf_coeff());
    }
}