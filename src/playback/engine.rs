use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Stopped,
    Playing,
    Paused,
}

pub trait Engine: Send + Sync {
    fn load(&mut self, path: &Path) -> Result<(), String>;
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn seek(&mut self, position: Duration);
    fn position(&self) -> Duration;
    fn state(&self) -> State;
}