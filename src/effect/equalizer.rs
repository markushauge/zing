use crate::graph::{Node, StreamInfo};
use eframe::egui;
use ringbuf::{Consumer, Producer, RingBuffer};

#[derive(Clone)]
pub enum Band {
    Peaking { frequency: f32, q: f32, gain: f32 },
}

pub struct Equalizer {
    bands: Vec<Band>,
    producer: Option<Producer<Message>>,
}

pub struct EqualizerNode {
    bands: Vec<Band>,
    filters: Option<Vec<BiQuadFilter>>,
    consumer: Consumer<Message>,
}

pub enum Message {
    UpdateBand { id: usize, band: Band },
}

impl Equalizer {
    pub fn new(bands: Vec<Band>) -> Self {
        Self {
            bands,
            producer: None,
        }
    }

    pub fn node(&mut self) -> EqualizerNode {
        let (producer, consumer) = RingBuffer::new(10).split();
        self.producer = Some(producer);

        EqualizerNode {
            bands: self.bands.clone(),
            filters: None,
            consumer,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        let bands = &mut self.bands;
        let producer = &mut self.producer;

        ui.horizontal(|ui| {
            for (id, band) in &mut bands.iter_mut().enumerate() {
                let band_clone = band.clone();

                ui.vertical(|ui| match band {
                    Band::Peaking { frequency, q, gain } => {
                        let gain_slider = egui::Slider::new(gain, -10.0..=10.0)
                            .orientation(egui::SliderOrientation::Vertical);

                        if ui.add(gain_slider).changed() {
                            if let Some(producer) = producer {
                                let _ = producer.push(Message::UpdateBand {
                                    id,
                                    band: band_clone.clone(),
                                });
                            }
                        }

                        let frequency_drag_value = egui::DragValue::new(frequency)
                            .clamp_range(0.0..=20_000.0)
                            .speed(10);

                        if ui.add(frequency_drag_value).changed() {
                            if let Some(producer) = producer {
                                let _ = producer.push(Message::UpdateBand {
                                    id,
                                    band: band_clone.clone(),
                                });
                            }
                        }

                        let q_drag_value =
                            egui::DragValue::new(q).clamp_range(0.1..=10.0).speed(0.01);

                        if ui.add(q_drag_value).changed() {
                            if let Some(producer) = producer {
                                let _ = producer.push(Message::UpdateBand {
                                    id,
                                    band: band_clone.clone(),
                                });
                            }
                        }
                    }
                });
            }
        });
    }
}

impl Node for EqualizerNode {
    fn read(&mut self, buffer: &mut [f32], info: &StreamInfo) {
        let filters = match self.filters.as_mut() {
            Some(filters) => filters,
            None => self.filters.insert(
                self.bands
                    .iter()
                    .map(|b| BiQuadFilter::new(Coefficients::from(b, info.sample_rate)))
                    .collect::<Vec<_>>(),
            ),
        };

        while let Some(message) = self.consumer.pop() {
            match message {
                Message::UpdateBand { id, band } => {
                    filters[id].coefficients = Coefficients::from(&band, info.sample_rate);
                }
            }
        }

        for filter in filters {
            filter.read(buffer, info);
        }
    }
}

#[derive(Debug)]
struct Coefficients {
    a0: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    a4: f32,
}

impl Coefficients {
    fn new(aa0: f32, aa1: f32, aa2: f32, b0: f32, b1: f32, b2: f32) -> Self {
        Self {
            a0: b0 / aa0,
            a1: b1 / aa0,
            a2: b2 / aa0,
            a3: aa1 / aa0,
            a4: aa2 / aa0,
        }
    }

    fn from(band: &Band, sample_rate: f32) -> Self {
        match band {
            Band::Peaking { frequency, q, gain } => {
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
        }
    }
}

struct State {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

struct BiQuadFilter {
    coefficients: Coefficients,
    state: State,
}

impl BiQuadFilter {
    fn new(coefficients: Coefficients) -> Self {
        Self {
            coefficients,
            state: State {
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            },
        }
    }
}

impl Node for BiQuadFilter {
    fn read(&mut self, buffer: &mut [f32], _: &StreamInfo) {
        let BiQuadFilter {
            coefficients: Coefficients { a0, a1, a2, a3, a4 },
            state: State { x1, x2, y1, y2 },
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
