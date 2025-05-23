use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::StringParams;

#[derive(Lens)]
struct Data {
    params: Arc<StringParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (200, 150))
}

pub(crate) fn create(
    params: Arc<StringParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone()
        }
        .build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "String Resonator")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Gain");
            //Knob::new(cx, Data::params, |params| &params.dry, true);
            ParamSlider::new(cx, Data::params, |params| &params.dry);
            ParamSlider::new(cx, Data::params, |params| &params.wet);
            
            //ParamSlider::new(cx, Data::params, |params| &params.base);
            
            //ParamSlider::new(cx, Data::params, |params| &params.lfo_rate);
            //ParamSlider::new(cx, Data::params, |params| &params.lfo_depth);
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));

        ResizeHandle::new(cx);
    })
}