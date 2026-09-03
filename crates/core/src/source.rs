use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Source {
    Track(Track),
    List(List),
    Folder(Folder),
    Clip(Clip),
}

#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub len: Option<Duration>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub start: Duration,
    pub end: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct List {
    pub children: Vec<Source>,
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub path: PathBuf,
    pub recursive: bool,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub inner: Box<Source>,
    pub start: Duration,
    pub end: Option<Duration>,
}

impl Source {
    pub fn track(path: impl Into<PathBuf>) -> Source {
        Source::Track(Track {
            path: path.into(),
            len: None,
            segments: Vec::new(),
        })
    }

    pub fn segment(path: impl Into<PathBuf>, start: Duration, end: Option<Duration>) -> Source {
        Source::Track(Track {
            path: path.into(),
            len: None,
            segments: vec![Segment { start, end }],
        })
    }
}
