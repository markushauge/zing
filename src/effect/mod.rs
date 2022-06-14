pub use self::equalizer::Band;
use crate::graph::{Node, StreamInfo};
use eframe::egui;

mod equalizer;
mod gain;

pub fn create_gain(gain: f32) -> Effect {
    Effect::Gain(gain::Gain::new(gain))
}

pub fn create_equalizer(bands: Vec<Band>) -> Effect {
    Effect::Equalizer(equalizer::Equalizer::new(bands))
}

pub enum Effect {
    Gain(gain::Gain),
    Equalizer(equalizer::Equalizer),
}

impl Effect {
    pub fn node(&mut self) -> EffectNode {
        match self {
            Self::Gain(gain) => EffectNode::Gain(gain.node()),
            Self::Equalizer(equalizer) => EffectNode::Equalizer(equalizer.node()),
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        match self {
            Self::Gain(gain) => gain.update(ui),
            Self::Equalizer(equalizer) => equalizer.update(ui),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Gain(_) => "Gain",
            Self::Equalizer(_) => "Equalizer",
        }
    }
}

pub enum EffectNode {
    Gain(gain::GainNode),
    Equalizer(equalizer::EqualizerNode),
}

impl Node for EffectNode {
    fn read(&mut self, buffer: &mut [f32], info: &StreamInfo) {
        match self {
            Self::Gain(gain) => gain.read(buffer, info),
            Self::Equalizer(equalizer) => equalizer.read(buffer, info),
        }
    }
}
