use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize)]
pub struct Settings {
    pub mono: bool,
    pub latency: f64,
    pub effects: Vec<Effect>,
}

impl Settings {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Effect {
    Gain { gain: f32 },
    Equalizer { bands: Vec<Band> },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Band {
    LowPass { cutoff_frequency: f64, q: f64 },
    HighPass { cutoff_frequency: f64, q: f64 },
}
