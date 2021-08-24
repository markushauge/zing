mod effect;
mod graph;
mod settings;

use self::settings::Settings;
use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, StreamConfig, StreamError,
};
use effect::EffectNode;
use graph::{Graph, InputNode, Node};
use ringbuf::RingBuffer;

const SETTINGS_PATH: &str = "settings.json";

fn find_device(host: &Host, name: &str) -> Option<Device> {
    for device in host.devices().ok()? {
        if device.name().ok()?.contains(name) {
            return Some(device);
        }
    }

    None
}

fn main() -> Result<()> {
    let settings = Settings::read(SETTINGS_PATH)?;
    let host = cpal::default_host();

    let input = settings
        .devices
        .as_ref()
        .and_then(|devices| devices.input.as_ref())
        .and_then(|name| find_device(&host, name))
        .or_else(|| host.default_input_device())
        .expect("no input device available");

    let output = settings
        .devices
        .as_ref()
        .and_then(|devices| devices.output.as_ref())
        .and_then(|name| find_device(&host, name))
        .or_else(|| host.default_output_device())
        .expect("no output device available");

    let mut config: StreamConfig = input.default_input_config()?.into();

    if settings.mono {
        config.channels = 1;
    }

    let sample_rate = config.sample_rate.0 as f64;
    let latency_frames = (settings.latency / 1_000.0) * sample_rate;
    let latency_samples = latency_frames as usize * config.channels as usize;
    let (mut producer, consumer) = RingBuffer::new(latency_samples * 2).split();

    for _ in 0..latency_samples {
        producer.push(0.0).unwrap();
    }

    let input_callback = move |buffer: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in buffer {
            producer.push(sample).unwrap();
        }
    };

    let effect_nodes: Vec<_> = settings
        .effects
        .iter()
        .map(|e| EffectNode::from(e, sample_rate))
        .collect();

    let input_node = InputNode::new(consumer);
    let mut graph = Graph::new(input_node, effect_nodes);

    let output_callback = move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
        graph.read(buffer);
    };

    let err_callback = |err: StreamError| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let input_stream = input.build_input_stream(&config, input_callback, err_callback)?;
    let output_stream = output.build_output_stream(&config, output_callback, err_callback)?;

    input_stream.play()?;
    output_stream.play()?;

    loop {
        std::thread::yield_now();
    }
}
