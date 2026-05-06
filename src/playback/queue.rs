use crate::media::Track;

pub struct Queue {
    tracks: Vec<Track>,
    cursor: usize,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            cursor: 0,
        }
    }

    pub fn push(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn set_cursor(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.cursor = index;
        }
    }

    pub fn cursor(&self) -> Option<usize> {
        if self.tracks.is_empty() {
            None
        } else {
            Some(self.cursor)
        }
    }

    pub fn current(&self) -> Option<&Track> {
        if self.tracks.is_empty() {
            None
        } else {
            self.tracks.get(self.cursor)
        }
    }

    pub fn advance(&mut self) -> Option<&Track> {
        if self.tracks.is_empty() {
            return None;
        }
        self.cursor = (self.cursor + 1) % self.tracks.len();
        self.tracks.get(self.cursor)
    }

    pub fn retreat(&mut self) -> Option<&Track> {
        if self.tracks.is_empty() {
            return None;
        }
        self.cursor = self.cursor.checked_sub(1).unwrap_or(self.tracks.len() - 1);
        self.tracks.get(self.cursor)
    }
}
