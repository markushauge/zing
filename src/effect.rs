use super::graph::Node;
use super::settings::{Band, Effect};

pub enum EffectNode {
    Gain(f32),
    Equalizer(Vec<BiQuadFilter>),
}

impl EffectNode {
    pub fn from(effect: &Effect, sample_rate: f32) -> Self {
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
    a0: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    a4: f32,

    // State
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiQuadFilter {
    fn from(band: &Band, sample_rate: f32) -> Self {
        match band {
            Band::LowPass { frequency, q } => Self::low_pass(sample_rate, *frequency, *q),
            Band::HighPass { frequency, q } => Self::high_pass(sample_rate, *frequency, *q),
            Band::Peaking { frequency, q, gain } => {
                Self::peaking(sample_rate, *frequency, *q, *gain)
            }
            Band::Notch { frequency, q } => Self::notch(sample_rate, *frequency, *q),
            Band::LowShelf {
                frequency,
                slope,
                gain,
            } => Self::low_shelf(sample_rate, *frequency, *slope, *gain),
            Band::HighShelf {
                frequency,
                slope,
                gain,
            } => Self::high_shelf(sample_rate, *frequency, *slope, *gain),
        }
    }

    fn new(aa0: f32, aa1: f32, aa2: f32, b0: f32, b1: f32, b2: f32) -> Self {
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

    fn low_pass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * q);

        let aa0 = 1.0 + alpha;
        let aa1 = -2.0 * cosw0;
        let aa2 = 1.0 - alpha;
        let b0 = (1.0 - cosw0) / 2.0;
        let b1 = 1.0 - cosw0;
        let b2 = (1.0 - cosw0) / 2.0;

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }

    fn high_pass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * q);

        let aa0 = 1.0 + alpha;
        let aa1 = -2.0 * cosw0;
        let aa2 = 1.0 - alpha;
        let b0 = (1.0 + cosw0) / 2.0;
        let b1 = -1.0 - cosw0;
        let b2 = (1.0 + cosw0) / 2.0;

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }

    fn peaking(sample_rate: f32, frequency: f32, q: f32, gain: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * q);
        let a = f32::powf(10.0, gain / 40.0);

        let aa0 = 1.0 + alpha / a;
        let aa1 = -2.0 * cosw0;
        let aa2 = 1.0 - alpha / a;
        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cosw0;
        let b2 = 1.0 - alpha * a;

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }

    fn notch(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let alpha = sinw0 / (2.0 * q);

        let aa0 = 1.0 + alpha;
        let aa1 = -2.0 * cosw0;
        let aa2 = 1.0 - alpha;
        let b0 = 1.0;
        let b1 = -2.0 * cosw0;
        let b2 = 1.0;

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }

    fn low_shelf(sample_rate: f32, frequency: f32, slope: f32, gain: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let a = f32::powf(10.0, gain / 40.0);
        let alpha = sinw0 / 2.0 * ((a + 1.0 / a) * (1.0 / slope - 1.0) + 2.0).sqrt();
        let temp = 2.0 * a.sqrt() * alpha;

        let aa0 = (a + 1.0) + (a - 1.0) * cosw0 + temp;
        let aa1 = -2.0 * ((a - 1.0) + (a + 1.0) * cosw0);
        let aa2 = (a + 1.0) + (a - 1.0) * cosw0 - temp;
        let b0 = a * ((a + 1.0) - (a - 1.0) * cosw0 + temp);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cosw0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cosw0 - temp);

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }

    fn high_shelf(sample_rate: f32, frequency: f32, slope: f32, gain: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let cosw0 = w0.cos();
        let sinw0 = w0.sin();
        let a = f32::powf(10.0, gain / 40.0);
        let alpha = sinw0 / 2.0 * ((a + 1.0 / a) * (1.0 / slope - 1.0) + 2.0).sqrt();
        let temp = 2.0 * a.sqrt() * alpha;

        let aa0 = (a + 1.0) - (a - 1.0) * cosw0 + temp;
        let aa1 = 2.0 * ((a - 1.0) - (a + 1.0) * cosw0);
        let aa2 = (a + 1.0) - (a - 1.0) * cosw0 - temp;
        let b0 = a * ((a + 1.0) + (a - 1.0) * cosw0 + temp);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cosw0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cosw0 - temp);

        Self::new(aa0, aa1, aa2, b0, b1, b2)
    }
}

impl Node for BiQuadFilter {
    fn read(&mut self, buffer: &mut [f32]) {
        let BiQuadFilter {
            a0,
            a1,
            a2,
            a3,
            a4,
            x1,
            x2,
            y1,
            y2,
        } = self;

        for sample in buffer {
            let result = *a0 * *sample + *a1 * *x1 + *a2 * *x2 - *a3 * *y1 - *a4 * *y2;
            *x2 = *x1;
            *x1 = *sample;
            *y2 = *y1;
            *y1 = result;
            *sample = result;
        }
    }
}
