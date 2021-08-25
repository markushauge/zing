use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize)]
pub struct Settings {
    pub devices: Option<Devices>,
    pub mono: bool,
    pub latency: f32,
    pub effects: Vec<Effect>,
}

impl Settings {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

#[derive(Deserialize)]
pub struct Devices {
    pub input: Option<String>,
    pub output: Option<String>,
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
    LowPass {
        frequency: f32,
        q: f32,
    },
    HighPass {
        frequency: f32,
        q: f32,
    },
    Peaking {
        frequency: f32,
        q: f32,
        gain: f32,
    },
    Notch {
        frequency: f32,
        q: f32,
    },
    LowShelf {
        frequency: f32,
        slope: f32,
        gain: f32,
    },
    HighShelf {
        frequency: f32,
        slope: f32,
        gain: f32,
    },
}
