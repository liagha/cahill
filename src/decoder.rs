use std::fs::File;
use std::time::Duration;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder as SymphoniaDecoder, DecoderOptions};
use symphonia::core::formats::{FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::StandardVisualKey;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

use crate::media::{CoverArt, MediaInfo};
use crate::playback::Decoder;

pub struct SymphoniaFileDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn SymphoniaDecoder>,
    sample_rate: u32,
    channels: u16,
    time_base: Option<TimeBase>,
    meta: MediaInfo,
    track_id: u32,
    position: Duration,
    duration: Duration,
}

impl Decoder for SymphoniaFileDecoder {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn next_frame(&mut self) -> Option<Vec<f32>> {
        let packet = match self.format.next_packet() {
            Ok(p) => p,
            Err(_) => return None,
        };

        let decoded = self.decoder.decode(&packet).ok()?;
        let spec = *decoded.spec();
        let count = decoded.frames() as u64;
        let mut buf = SampleBuffer::<f32>::new(count, spec);
        buf.copy_interleaved_ref(decoded);

        if let Some(tb) = self.time_base {
            let ts = packet.ts();
            let secs = ts as f64 * tb.numer as f64 / tb.denom as f64;
            self.position = Duration::from_secs_f64(secs);
        } else {
            let secs = packet.ts() as f64 / self.sample_rate as f64;
            self.position = Duration::from_secs_f64(secs);
        }

        Some(buf.samples().to_vec())
    }

    fn seek(&mut self, time: Duration) {
        let ts = if let Some(tb) = self.time_base {
            (time.as_secs_f64() * tb.denom as f64 / tb.numer as f64) as u64
        } else {
            (time.as_secs_f64() * self.sample_rate as f64) as u64
        };

        let seek_to = SeekTo::Time {
            track_id: Some(self.track_id),
            time: Time::from(ts),
        };

        if self.format.seek(SeekMode::Accurate, seek_to).is_ok() {
            self.position = time;
        }
    }

    fn position(&self) -> Option<Duration> {
        Some(self.position)
    }

    fn duration(&self) -> Option<Duration> {
        Some(self.duration)
    }

    fn metadata(&self) -> MediaInfo {
        self.meta.clone()
    }
}

pub fn open(path: &str) -> Result<SymphoniaFileDecoder, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let hint = Hint::new();
    let mut probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &Default::default(),
        &Default::default(),
    )?;

    let format = probed.format;
    let track = format.default_track().ok_or("no track")?;
    let track_id = track.id;
    let params = track.codec_params.clone();

    let sample_rate = params.sample_rate.unwrap_or(44100);
    let channels = params.channels.map(|c| c.count()).unwrap_or(2) as u16;
    let duration = params
        .n_frames
        .map(|frames| Duration::from_secs_f64(frames as f64 / sample_rate as f64))
        .unwrap_or(Duration::ZERO);

    let time_base = params.time_base;
    let decoder = symphonia::default::get_codecs().make(&params, &DecoderOptions::default())?;

    let mut title = String::new();
    let mut artist = String::new();
    let mut album = String::new();
    let mut cover = None;

    if let Some(rev) = probed.metadata.get().unwrap().current() {
        for tag in rev.tags() {
            let key = tag.key.to_lowercase();
            let val = tag.value.to_string();
            match key.as_str() {
                "title" => title = val,
                "artist" => artist = val,
                "album" => album = val,
                _ => {}
            }
        }

        if let Some(visual) = rev.visuals().iter().find(|v| v.usage == Some(StandardVisualKey::FrontCover)) {
            cover = Some(CoverArt {
                mime_type: visual.media_type.clone(),
                data: visual.data.to_vec(),
            });
        }
    }

    Ok(SymphoniaFileDecoder {
        format,
        decoder,
        sample_rate,
        channels,
        time_base,
        meta: MediaInfo {
            title,
            artist,
            album,
            duration,
            path: path.to_string(),
            cover,
        },
        track_id,
        position: Duration::ZERO,
        duration,
    })
}