use std::io;
use std::time::Duration;

pub trait MediaSource: Send + Sync {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn seek(&mut self, pos: u64) -> io::Result<u64>;
    fn byte_len(&self) -> Option<u64>;
}

pub trait Decoder: Send {
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn next_frame(&mut self) -> Option<Vec<f32>>;
    fn seek(&mut self, time: Duration);
    fn position(&self) -> Option<Duration>;
    fn duration(&self) -> Option<Duration>;
    fn metadata(&self) -> crate::media::MediaInfo;
}

pub trait AudioOutput: Send {
    fn play(&self);
    fn pause(&self);
    fn stop(&self);
    fn volume(&self, level: f32);
    fn push_samples(&self, samples: &[f32]);
}
