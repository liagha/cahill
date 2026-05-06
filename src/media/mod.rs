pub mod probe;

use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<Duration>,
    pub cover: Option<Vec<u8>>,
    pub path: PathBuf,
}

impl Meta {
    pub fn display_title(&self) -> &str {
        self.title
            .as_deref()
            .or_else(|| self.path.file_stem().and_then(|s| s.to_str()))
            .unwrap_or("Unknown")
    }

    pub fn display_artist(&self) -> &str {
        self.artist.as_deref().unwrap_or("Unknown Artist")
    }
}

pub trait Playable: Send + Sync {
    fn meta(&self) -> &Meta;
    fn path(&self) -> &std::path::Path;
}

pub struct Track {
    pub meta: Meta,
}

impl Playable for Track {
    fn meta(&self) -> &Meta {
        &self.meta
    }

    fn path(&self) -> &std::path::Path {
        &self.meta.path
    }
}