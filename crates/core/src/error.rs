use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

pub enum Error {
    Missing { path: PathBuf },
    NotFolder { path: PathBuf },
    Empty { path: PathBuf },
    Clip { start: Duration, end: Option<Duration> },
    Reversed { start: Duration, end: Duration },
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Missing { path } => write!(f, "missing {:?}", path),
            Error::NotFolder { path } => write!(f, "not a folder {:?}", path),
            Error::Empty { path } => write!(f, "no audio under {:?}", path),
            Error::Clip { start, end } => {
                write!(f, "clip {:?}..{:?} out of range", start, end)
            }
            Error::Reversed { start, end } => {
                write!(f, "clip start {:?} after end {:?}", start, end)
            }
            Error::Other(msg) => f.write_str(msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
