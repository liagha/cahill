use crate::media::Track;

pub struct Queue {
    tracks: Vec<Track>,
    cursor: Option<usize>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            cursor: None,
        }
    }

    pub fn push(&mut self, track: Track) {
        self.tracks.push(track);
        if self.cursor.is_none() {
            self.cursor = Some(0);
        }
    }

    pub fn current(&self) -> Option<&Track> {
        self.cursor.and_then(|i| self.tracks.get(i))
    }

    pub fn next(&mut self) -> Option<&Track> {
        let next = self.cursor.map(|i| i + 1).filter(|&i| i < self.tracks.len());
        self.cursor = next;
        self.current()
    }

    pub fn prev(&mut self) -> Option<&Track> {
        let prev = self.cursor.and_then(|i| i.checked_sub(1));
        self.cursor = prev;
        self.current()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }
}