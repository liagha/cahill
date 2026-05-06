use crate::media::MediaInfo;
use crate::playback::Decoder;
use std::path::Path;

pub fn scan_directory(dir: &str) -> Vec<MediaInfo> {
    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(
                        ext.to_lowercase().as_str(),
                        "mp3" | "mp4" | "m4a" | "flac" | "ogg" | "wav"
                    ) {
                        if let Ok(info) = extract_metadata(&path) {
                            list.push(info);
                        }
                    }
                }
            }
        }
    }
    list
}

fn extract_metadata(path: &Path) -> Result<MediaInfo, Box<dyn std::error::Error>> {
    let decoder = crate::decoder::open(path.to_str().unwrap())?;
    Ok(decoder.metadata())
}