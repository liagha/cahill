use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

use super::engine::{Engine, State};

pub struct RodioEngine {
    player: Player,
    _stream: Arc<MixerDeviceSink>,
    state: State,
    total_duration: Duration,
    current_path: Option<std::path::PathBuf>,
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
            total_duration: Duration::ZERO,
            current_path: None,
        })
    }

    fn reopen_at(&mut self, position: Duration) -> Result<(), String> {
        if let Some(ref path) = self.current_path {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let source = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
            let dur = source.total_duration().unwrap_or(Duration::ZERO);
            self.total_duration = dur;
            self.player.stop();
            self.player.append(source);
            self.player.play();
            self.player.try_seek(position).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("no track loaded".into())
        }
    }
}

impl Engine for RodioEngine {
    fn load(&mut self, path: &Path) -> Result<Duration, String> {
        self.current_path = Some(path.to_path_buf());
        let file = File::open(path).map_err(|e| e.to_string())?;
        let source = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
        let dur = source.total_duration().unwrap_or(Duration::ZERO);
        self.total_duration = dur;
        self.player.stop();
        self.player.append(source);
        self.player.play();
        self.state = State::Playing;
        Ok(dur)
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
        let forward_seek = position >= self.player.get_pos();
        let result = self.player.try_seek(position);
        if result.is_err() || (!forward_seek && self.player.get_pos() > position + Duration::from_millis(100)) {
            let _ = self.reopen_at(position);
        }
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

    fn duration(&self) -> Duration {
        self.total_duration
    }
}