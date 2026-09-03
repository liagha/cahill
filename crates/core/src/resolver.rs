use std::fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::source::{Folder, Track};

pub trait Resolver {
    fn list(&self, folder: &Folder) -> Result<Vec<Track>>;
}

pub struct Fs;

impl Resolver for Fs {
    fn list(&self, folder: &Folder) -> Result<Vec<Track>> {
        let mut paths = Vec::new();
        walk(&folder.path, folder.recursive, &folder.extensions, &mut paths)?;
        if paths.is_empty() {
            return Err(Error::Empty {
                path: folder.path.clone(),
            });
        }
        Ok(paths
            .into_iter()
            .map(|path| Track {
                path,
                len: None,
                segments: Vec::new(),
            })
            .collect())
    }
}

fn audio(path: &Path, extensions: &[String]) -> bool {
    extensions
        .iter()
        .any(|ext| path.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)))
}

fn walk(path: &Path, recursive: bool, extensions: &[String], out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let meta = fs::metadata(path).map_err(|_| Error::Missing {
        path: path.to_path_buf(),
    })?;
    if !meta.is_dir() {
        return Err(Error::NotFolder {
            path: path.to_path_buf(),
        });
    }
    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|_| Error::Missing {
            path: path.to_path_buf(),
        })?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let child = entry.path();
        if child.is_dir() {
            if recursive {
                walk(&child, recursive, extensions, out)?;
            }
        } else if audio(&child, extensions) {
            out.push(child);
        }
    }
    Ok(())
}
