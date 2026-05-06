use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct CoverArt {
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub path: String,
    pub cover: Option<CoverArt>,
}