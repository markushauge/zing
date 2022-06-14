use crate::graph::{Node, StreamInfo};
use eframe::egui;
use ringbuf::{Consumer, Producer, RingBuffer};

pub struct Gain {
    gain: f32,
    producer: Option<Producer<Message>>,
}

pub struct GainNode {
    gain: f32,
    consumer: Consumer<Message>,
}

enum Message {
    UpdateGain { gain: f32 },
}

impl Gain {
    pub fn new(gain: f32) -> Self {
        Self {
            gain,
            producer: None,
        }
    }

    pub fn node(&mut self) -> GainNode {
        let (producer, consumer) = RingBuffer::new(10).split();
        self.producer = Some(producer);

        GainNode {
            gain: self.gain,
            consumer,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        let slider = egui::Slider::new(&mut self.gain, 0.0..=4.0);

        if ui.add(slider).changed() {
            if let Some(producer) = self.producer.as_mut() {
                let _ = producer.push(Message::UpdateGain { gain: self.gain });
            }
        }
    }
}

impl Node for GainNode {
    fn read(&mut self, buffer: &mut [f32], _: &StreamInfo) {
        while let Some(message) = self.consumer.pop() {
            match message {
                Message::UpdateGain { gain } => {
                    self.gain = gain;
                }
            };
        }

        for sample in buffer {
            *sample *= self.gain;
        }
    }
}
