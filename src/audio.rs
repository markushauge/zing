use crate::{
    effect::{Effect, EffectMessage, EffectNode},
    graph::{Graph, InputNode, Node},
};
use anyhow::{anyhow, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, Stream, StreamConfig, StreamError,
};
use ringbuf::{Producer, RingBuffer};

pub enum Message {
    Add { effect: Effect },
    Remove { id: usize },
    Update { id: usize, message: EffectMessage },
}

pub struct Audio {
    host: Host,
    latency: f32,
}

pub struct AudioSession {
    input: DeviceStream,
    output: DeviceStream,
    producer: Producer<Message>,
}

impl AudioSession {
    pub fn input(&self) -> Result<String> {
        Ok(self.input.device.name()?)
    }

    pub fn output(&self) -> Result<String> {
        Ok(self.output.device.name()?)
    }

    pub fn dispatch(&mut self, message: Message) {
        let _ = self.producer.push(message);
    }
}

struct DeviceStream {
    device: Device,
    _stream: Stream,
}

impl Audio {
    pub fn new(latency: f32) -> Self {
        let host = cpal::default_host();
        Self { host, latency }
    }

    pub fn inputs(&self) -> Result<Vec<String>> {
        Ok(self
            .host
            .input_devices()?
            .filter_map(|d| d.name().ok())
            .collect())
    }

    pub fn outputs(&self) -> Result<Vec<String>> {
        Ok(self
            .host
            .output_devices()?
            .filter_map(|d| d.name().ok())
            .collect())
    }

    pub fn session(&self, input: &str, output: &str, effects: &[Effect]) -> Result<AudioSession> {
        let input = find_input_device(&self.host, input)
            .or_else(|| self.host.default_input_device())
            .ok_or_else(|| anyhow!("no input device available"))?;

        let output = find_output_device(&self.host, output)
            .or_else(|| self.host.default_output_device())
            .ok_or_else(|| anyhow!("no output device available"))?;

        let config: StreamConfig = input.default_input_config()?.into();
        let sample_rate = config.sample_rate.0 as f32;
        let latency_frames = (self.latency / 1_000.0) * sample_rate;
        let latency_samples = latency_frames as usize * config.channels as usize;
        let (mut sample_producer, sample_consumer) = RingBuffer::new(latency_samples * 2).split();
        sample_producer.push_iter(&mut std::iter::repeat(0.0).take(latency_samples));

        let input_callback = move |buffer: &[f32], _: &cpal::InputCallbackInfo| {
            if sample_producer.push_slice(buffer) < buffer.len() {
                eprintln!("Output stream fell behind");
            }
        };

        let effect_nodes: Vec<_> = effects
            .iter()
            .map(|e| EffectNode::from(e, sample_rate))
            .collect();

        let input_node = InputNode::new(sample_consumer);
        let mut graph = Graph::new(input_node, effect_nodes);
        let (producer, mut consumer) = RingBuffer::new(self.latency as usize).split();

        let output_callback = move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            consumer.pop_each(
                |m| {
                    match m {
                        Message::Add { effect } => {
                            graph.add_node(EffectNode::from(&effect, sample_rate))
                        }
                        Message::Remove { id } => graph.remove_node(id),
                        Message::Update { id, message } => {
                            if let (EffectNode::Gain(gain), EffectMessage::UpdateGain { volume }) =
                                (graph.node(id), message)
                            {
                                *gain = volume;
                            }
                        }
                    }

                    true
                },
                None,
            );

            graph.read(buffer);
        };

        let err_callback = |err: StreamError| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let input_stream = input.build_input_stream(&config, input_callback, err_callback)?;
        let output_stream = output.build_output_stream(&config, output_callback, err_callback)?;

        input_stream.play()?;
        output_stream.play()?;

        Ok(AudioSession {
            input: DeviceStream {
                device: input,
                _stream: input_stream,
            },
            output: DeviceStream {
                device: output,
                _stream: output_stream,
            },
            producer,
        })
    }
}

fn find_input_device(host: &Host, name: &str) -> Option<Device> {
    for device in host.input_devices().ok()? {
        if device.name().ok()?.contains(name) {
            return Some(device);
        }
    }

    None
}

fn find_output_device(host: &Host, name: &str) -> Option<Device> {
    for device in host.output_devices().ok()? {
        if device.name().ok()?.contains(name) {
            return Some(device);
        }
    }

    None
}
