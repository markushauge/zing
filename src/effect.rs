use super::graph::Node;
use super::settings::{Band, Effect};

pub enum EffectNode {
    Gain(f32),
    Equalizer(Vec<BiQuadFilter>),
}

impl EffectNode {
    pub fn from(effect: &Effect, sample_rate: f64) -> Self {
        match effect {
            Effect::Gain { gain } => EffectNode::Gain(*gain),
            Effect::Equalizer { bands } => EffectNode::Equalizer(
                bands
                    .iter()
                    .map(|b| BiQuadFilter::from(b, sample_rate))
                    .collect(),
            ),
        }
    }
}

impl Node for EffectNode {
    fn read(&mut self, buffer: &mut [f32]) {
        match self {
            EffectNode::Gain(gain) => {
                for sample in buffer {
                    *sample *= *gain;
                }
            }
            EffectNode::Equalizer(filters) => {
                for filter in filters {
                    filter.read(buffer);
                }
            }
        }
    }
}

pub struct BiQuadFilter {
    // Coefficients
    a0: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    a4: f64,

    // State
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiQuadFilter {
    fn from(band: &Band, sample_rate: f64) -> Self {
        match band {
            Band::LowPass {
                cutoff_frequency,
                q,
            } => Self::low_pass(sample_rate, *cutoff_frequency, *q),
            Band::HighPass {
                cutoff_frequency,
                q,
            } => Self::high_pass(sample_rate, *cutoff_frequency, *q),
        }
    }

    fn new(aa0: f64, aa1: f64, aa2: f64, b0: f64, b1: f64, b2: f64) -> Self {
        Self {
            a0: b0 / aa0,
            a1: b1 / aa0,
            a2: b2 / aa0,
            a3: aa1 / aa0,
            a4: aa2 / aa0,

            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn low_pass(sample_rate: f64, cutoff_frequency: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * cutoff_frequency / sample_rate;
        let cosw0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let aa0 = 1.0 + alpha;
        let aa1 = -2.0 * cosw0;
        let aa2 = 1.0 - alpha;
        let b0 = (1.0 - cosw0) / 2.0;
        let b1 = 1.0 - cosw0;
        let b2 = (1.0 - cosw0) / 2.0;

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }

    fn high_pass(sample_rate: f64, cutoff_frequency: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * cutoff_frequency / sample_rate;
        let cosw0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let aa0 = 1.0 + alpha;
        let aa1 = -2.0 * cosw0;
        let aa2 = 1.0 - alpha;
        let b0 = (1.0 + cosw0) / 2.0;
        let b1 = -1.0 - cosw0;
        let b2 = (1.0 + cosw0) / 2.0;

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }
}

impl Node for BiQuadFilter {
    fn read(&mut self, buffer: &mut [f32]) {
        for sample in buffer {
            let result =
                self.a0 as f32 * *sample + self.a1 as f32 * self.x1 + self.a2 as f32 * self.x2
                    - self.a3 as f32 * self.y1
                    - self.a4 as f32 * self.y2;

            self.x2 = self.x1;
            self.x1 = *sample;

            self.y2 = self.y1;
            self.y1 = result;

            *sample = result;
        }
    }
}
