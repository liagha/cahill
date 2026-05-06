use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use super::engine::{Engine, State};

pub struct RodioEngine {
    player: Player,
    _stream: Arc<MixerDeviceSink>,
    state: State,
}

impl RodioEngine {
    pub fn new() -> Result<Self, String> {
        let stream = Arc::new(
            DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?
        );

        let player = Player::connect_new(stream.mixer());
        Ok(Self {
            player,
            _stream: stream,
            state: State::Stopped,
        })
    }
}

impl Engine for RodioEngine {
    fn load(&mut self, path: &Path) -> Result<(), String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let source = Decoder::new(file).map_err(|e| e.to_string())?;
        self.player.stop();
        self.player.append(source);
        self.player.play();
        self.state = State::Playing;
        Ok(())
    }

    fn play(&mut self) {
        self.player.play();
        self.state = State::Playing;
    }

    fn pause(&mut self) {
        self.player.pause();
        self.state = State::Paused;
    }

    fn seek(&mut self, position: Duration) {
        let _ = self.player.try_seek(position);
    }

    fn position(&self) -> Duration {
        self.player.get_pos()
    }

    fn finished(&self) -> bool {
        self.player.empty()
    }

    fn state(&self) -> State {
        self.state.clone()
    }
}