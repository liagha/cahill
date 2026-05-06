use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

use super::{Meta, Track};

pub fn load(path: &Path) -> Option<Track> {
    let file = std::fs::File::open(path).ok()?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut probed = get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;

    let mut meta = Meta {
        path: path.to_path_buf(),
        ..Default::default()
    };

    if let Some(revision) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
        for tag in revision.tags() {
            match tag.std_key {
                Some(StandardTagKey::TrackTitle) => {
                    meta.title = Some(tag.value.to_string());
                }
                Some(StandardTagKey::Artist) => {
                    meta.artist = Some(tag.value.to_string());
                }
                Some(StandardTagKey::Album) => {
                    meta.album = Some(tag.value.to_string());
                }
                _ => {}
            }
        }

        for visual in revision.visuals() {
            if meta.cover.is_none() {
                meta.cover = Some(visual.data.to_vec());
            }
        }
    }

    let format = probed.format.as_mut();
    if let Some(track) = format.default_track() {
        if let Some(tb) = track.codec_params.time_base {
            if let Some(frames) = track.codec_params.n_frames {
                let time = tb.calc_time(frames);
                meta.duration = Some(std::time::Duration::from_secs_f64(
                    time.seconds as f64 + time.frac,
                ));
            }
        }
    }

    Some(Track { meta })
}