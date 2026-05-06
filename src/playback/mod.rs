pub mod engine;
pub mod queue;

#[cfg(not(target_arch = "wasm32"))]
pub mod rodio;

use std::path::Path;
use std::time::Duration;

use engine::{Engine, State};
use queue::Queue;

#[cfg(not(target_arch = "wasm32"))]
use self::rodio::RodioEngine;

pub struct Player {
    engine: Box<dyn Engine>,
    pub queue: Queue,
}

impl Player {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            engine: Box::new(RodioEngine::new()?),
            queue: Queue::new(),
        })
    }

    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        self.engine.load(path)
    }

    pub fn play(&mut self) {
        self.engine.play();
    }

    pub fn pause(&mut self) {
        self.engine.pause();
    }

    pub fn toggle(&mut self) {
        match self.engine.state() {
            State::Playing => self.pause(),
            _ => self.play(),
        }
    }

    pub fn seek(&mut self, position: Duration) {
        self.engine.seek(position);
    }

    pub fn position(&self) -> Duration {
        self.engine.position()
    }

    pub fn state(&self) -> State {
        self.engine.state()
    }
}