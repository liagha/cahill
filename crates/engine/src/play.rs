use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use symphonia::core::audio::{Channels, SampleBuffer, SignalSpec};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use rubato::{FftFixedIn, Resampler};

use crate::resolve::Resolved;
use crate::ring::Ring;

#[derive(Debug, Clone, Copy)]
pub struct OutConfig {
    pub rate: u32,
    pub channels: u16,
}

pub fn region(
    path: &Path,
    start: Duration,
    end: Option<Duration>,
    config: OutConfig,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(extension);
    }
    let mut probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let track = probed
        .format
        .default_track()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no audio track"))?;
    let track_id = track.id;
    let params = track.codec_params.clone();
    let native_rate = params
        .sample_rate
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no sample rate"))?;
    let channels = params
        .channels
        .unwrap_or_else(|| Channels::FRONT_LEFT | Channels::FRONT_RIGHT);
    let channel_count = channels.count() as u16;
    let options = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs().make(&params, &options)?;

    let start_frames = (start.as_secs_f64() * f64::from(native_rate)) as usize;
    let end_frames = end
        .map(|e| (e.as_secs_f64() * f64::from(native_rate)) as usize)
        .unwrap_or(usize::MAX);

    let spec = SignalSpec {
        rate: native_rate,
        channels,
    };
    let mut sample_buf =
        SampleBuffer::<f32>::new(native_rate as u64 * channel_count as u64, spec);
    let mut decoded_frames = 0usize;
    let mut native: Vec<f32> = Vec::new();
    while let Ok(packet) = probed.format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SError::DecodeError(_)) | Err(SError::IoError(_)) => continue,
            Err(_) => break,
        };
        sample_buf.copy_interleaved_ref(decoded);
        let samples = sample_buf.samples();
        let frames = samples.len() / channel_count as usize;
        let begin = decoded_frames;
        let end = begin + frames;
        if end > start_frames {
            let keep = end.min(end_frames).saturating_sub(begin.max(start_frames));
            if keep > 0 {
                let from = begin.max(start_frames).saturating_sub(begin);
                let s0 = from * channel_count as usize;
                let s1 = (from + keep) * channel_count as usize;
                native.extend_from_slice(&samples[s0..s1.min(samples.len())]);
            }
        }
        if end >= end_frames {
            break;
        }
        decoded_frames = end;
    }

    if native_rate == config.rate && channel_count == config.channels {
        return Ok(native);
    }

    let remixed = remix(&native, channel_count, config.channels);

    if native_rate == config.rate {
        return Ok(remixed);
    }

    let mut planar: Vec<Vec<f32>> = vec![Vec::new(); config.channels as usize];
    for (index, &sample) in remixed.iter().enumerate() {
        planar[index % config.channels as usize].push(sample);
    }
    let mut resampler = FftFixedIn::<f32>::new(
        native_rate as usize,
        config.rate as usize,
        1024,
        1,
        config.channels as usize,
    )?;
    let out_planar = resampler.process(&planar, None)?;
    let frames = out_planar[0].len();
    let mut out = Vec::with_capacity(frames * config.channels as usize);
    for frame in 0..frames {
        for channel in out_planar.iter() {
            out.push(channel[frame]);
        }
    }
    Ok(out)
}

fn remix(native: &[f32], from: u16, to: u16) -> Vec<f32> {
    if from == to {
        return native.to_vec();
    }
    let frames = native.len() / from as usize;
    let mut out = Vec::with_capacity(frames * to as usize);
    for frame in 0..frames {
        let base = frame * from as usize;
        for channel in 0..to as usize {
            let value = if to > from {
                native[base + (channel % from as usize)]
            } else {
                native[base..base + from as usize].iter().sum::<f32>() / from as f32
            };
            out.push(value);
        }
    }
    out
}

pub struct Player {
    _stream: cpal::Stream,
    _thread: std::thread::JoinHandle<()>,
    pub total: Duration,
}

pub fn play(resolved: &Resolved) -> Result<Player, Box<dyn std::error::Error>> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no output device"))?;
    let supported = device.default_output_config()?;
    let format = supported.sample_format();
    let config = supported.config();
    let cfg = OutConfig {
        rate: config.sample_rate.0,
        channels: config.channels,
    };
    let ring = Arc::new(Ring::new(config.sample_rate.0 as usize * config.channels as usize));

    let events = resolved.events.clone();
    let ring_out = ring.clone();
    let _thread = std::thread::spawn(move || {
        for event in &events {
            let region = region(
                &event.play.path,
                event.play.start,
                event.play.end,
                cfg,
            );
            match region {
                Ok(samples) => {
                    let mut offset = 0;
                    while offset < samples.len() {
                        let slice = &samples[offset..];
                        let pushed = if ring_out.space() >= 1024 {
                            ring_out.push(slice)
                        } else {
                            ring_out.wait_push(slice)
                        };
                        let pushed = pushed.max(1).min(slice.len());
                        offset += pushed;
                    }
                }
                Err(_) => break,
            }
        }
        ring_out.mark_done();
    });

    let stream = match format {
        cpal::SampleFormat::F32 => stream::<f32>(&device, &config, &ring)?,
        cpal::SampleFormat::F64 => stream::<f64>(&device, &config, &ring)?,
        cpal::SampleFormat::I16 => stream::<i16>(&device, &config, &ring)?,
        cpal::SampleFormat::U16 => stream::<u16>(&device, &config, &ring)?,
        cpal::SampleFormat::I32 => stream::<i32>(&device, &config, &ring)?,
        cpal::SampleFormat::U32 => stream::<u32>(&device, &config, &ring)?,
        cpal::SampleFormat::I8 => stream::<i8>(&device, &config, &ring)?,
        cpal::SampleFormat::U8 => stream::<u8>(&device, &config, &ring)?,
        other => return Err(format!("unsupported sample format {other:?}").into()),
    };
    stream.play()?;

    Ok(Player {
        _stream: stream,
        _thread,
        total: resolved.total,
    })
}

fn stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    ring: &Arc<Ring>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    use cpal::traits::DeviceTrait;
    let ring = ring.clone();
    let data = move |out: &mut [T], _info: &cpal::OutputCallbackInfo| {
        let mut tmp = vec![0.0f32; out.len()];
        let n = ring.pop(&mut tmp);
        for (index, slot) in out.iter_mut().enumerate() {
            let sample = if index < n { tmp[index] } else { 0.0 };
            *slot = T::from_sample(sample);
        }
    };
    let err = |e| eprintln!("cahill: audio stream error: {e}");
    let stream = device.build_output_stream(config, data, err, None)?;
    Ok(stream)
}
