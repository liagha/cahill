use std::sync::mpsc;
use std::time::Duration;
use crate::media::MediaInfo;
use crate::playback::{AudioOutput, Decoder};

#[derive(Clone, Debug, PartialEq)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration),
    Volume(f32),
    Load(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlayerEvent {
    State(PlayerState),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerState {
    pub playing: bool,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub media: Option<MediaInfo>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            playing: false,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 1.0,
            media: None,
        }
    }
}

pub struct Player {
    pub sender: mpsc::Sender<PlayerCommand>,
    receiver: mpsc::Receiver<PlayerEvent>,
    _handle: Option<std::thread::JoinHandle<()>>,
}

impl Player {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (evt_tx, evt_rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            run(cmd_rx, evt_tx);
        });
        Self {
            sender: cmd_tx,
            receiver: evt_rx,
            _handle: Some(handle),
        }
    }

    pub fn send(&self, cmd: PlayerCommand) {
        self.sender.send(cmd).ok();
    }

    pub fn try_recv(&self) -> Option<PlayerEvent> {
        self.receiver.try_recv().ok()
    }
}

fn run(cmd_rx: mpsc::Receiver<PlayerCommand>, evt_tx: mpsc::Sender<PlayerEvent>) {
    let mut state = PlayerState::default();
    let mut decoder: Option<Box<dyn Decoder>> = None;
    let mut output: Option<Box<dyn AudioOutput>> = None;

    loop {
        match cmd_rx.try_recv() {
            Ok(cmd) => match cmd {
                PlayerCommand::Load(path) => {
                    decoder = crate::decoder::open(&path)
                        .ok()
                        .map(|d| Box::new(d) as Box<dyn Decoder>);
                    if let Some(ref dec) = decoder {
                        state.media = Some(dec.metadata());
                        state.duration = dec.duration().unwrap_or(Duration::ZERO);
                        output = Some(crate::output::create_output(
                            dec.sample_rate(),
                            dec.channels(),
                        ));
                        output.as_ref().unwrap().volume(state.volume);
                    }
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
                PlayerCommand::Play => {
                    state.playing = true;
                    if let Some(ref out) = output {
                        out.play();
                    }
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
                PlayerCommand::Pause => {
                    state.playing = false;
                    if let Some(ref out) = output {
                        out.pause();
                    }
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
                PlayerCommand::Stop => {
                    state.playing = false;
                    state.position = Duration::ZERO;
                    if let Some(ref out) = output {
                        out.stop();
                    }
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
                PlayerCommand::Seek(pos) => {
                    if let Some(ref mut dec) = decoder {
                        dec.seek(pos);
                    }
                    state.position = pos;
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
                PlayerCommand::Volume(vol) => {
                    state.volume = vol;
                    if let Some(ref out) = output {
                        out.volume(vol);
                    }
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
            },
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break,
        }

        if state.playing {
            if let Some(ref mut dec) = decoder {
                if let Some(frame) = dec.next_frame() {
                    if let Some(ref out) = output {
                        out.push_samples(&frame);
                    }
                    state.position = dec.position().unwrap_or(state.position);
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                } else {
                    state.playing = false;
                    if let Some(ref out) = output {
                        out.stop();
                    }
                    evt_tx.send(PlayerEvent::State(state.clone())).ok();
                }
            }
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}