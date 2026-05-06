use crate::playback::AudioOutput;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct CpalOutput {
    stream: Mutex<Option<cpal::Stream>>,
    volume: Arc<Mutex<f32>>,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioOutput for CpalOutput {
    fn play(&self) {
        if let Ok(guard) = self.stream.lock() {
            if let Some(ref stream) = *guard {
                stream.play().ok();
            }
        }
    }

    fn pause(&self) {
        if let Ok(guard) = self.stream.lock() {
            if let Some(ref stream) = *guard {
                stream.pause().ok();
            }
        }
    }

    fn stop(&self) {
        if let Ok(guard) = self.stream.lock() {
            if let Some(ref stream) = *guard {
                stream.pause().ok();
            }
        }
    }

    fn volume(&self, level: f32) {
        *self.volume.lock().unwrap() = level;
    }

    fn push_samples(&self, samples: &[f32]) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.extend(samples);
        }
    }
}

fn find_matching_config(device: &cpal::Device, sample_rate: u32, channels: u16) -> Option<cpal::StreamConfig> {
    let configs = device.supported_output_configs().ok()?;
    for range in configs {
        if range.channels() as u16 == channels
            && range.min_sample_rate() <= sample_rate
            && range.max_sample_rate() >= sample_rate
        {
            return Some(cpal::StreamConfig {
                channels: channels as cpal::ChannelCount,
                sample_rate,
                buffer_size: cpal::BufferSize::Default,
            });
        }
    }
    None
}

pub fn create_output(sample_rate: u32, channels: u16) -> Box<dyn AudioOutput> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");

    let config = find_matching_config(&device, sample_rate, channels)
        .unwrap_or_else(|| device.default_output_config().expect("no config").config());

    let buffer = Arc::new(Mutex::new(VecDeque::new()));
    let buf = buffer.clone();
    let volume = Arc::new(Mutex::new(1.0f32));
    let vol = volume.clone();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let v = *vol.lock().unwrap();
                let mut buf = buf.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = buf.pop_front().unwrap_or(0.0) * v;
                }
            },
            |err| eprintln!("audio error: {}", err),
            None,
        )
        .ok();

    Box::new(CpalOutput {
        stream: Mutex::new(stream),
        volume,
        buffer,
    })
}