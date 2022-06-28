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
        let bands = &self.bands;

        let filters = self.filters.get_or_insert_with(|| {
            bands
                .iter()
                .map(|b| BiQuadFilter::new(b.coefficients(info.sample_rate)))
                .collect::<Vec<_>>()
        });

        while let Some(message) = self.consumer.pop() {
            match message {
                Message::UpdateBand { id, band } => {
                    filters[id].coefficients = band.coefficients(info.sample_rate);
                }
            }
        }

        for filter in filters {
            filter.read(buffer, info);
        }
    }
}

impl Band {
    fn coefficients(&self, sample_rate: f32) -> [f32; 5] {
        match self {
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

                [b0 / aa0, b1 / aa1, b2 / aa2, aa1 / aa0, aa2 / aa0]
            }
        }
    }
}

struct BiQuadFilter {
    coefficients: [f32; 5],
    state: [f32; 4],
}

impl BiQuadFilter {
    fn new(coefficients: [f32; 5]) -> Self {
        Self {
            coefficients,
            state: Default::default(),
        }
    }
}

impl Node for BiQuadFilter {
    fn read(&mut self, buffer: &mut [f32], _: &StreamInfo) {
        let BiQuadFilter {
            coefficients: [a0, a1, a2, a3, a4],
            state: [x1, x2, y1, y2],
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
