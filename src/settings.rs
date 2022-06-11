use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize)]
pub struct Settings {
    pub latency: f32,
}

impl Settings {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self { latency: 256.0 }
    }
}
