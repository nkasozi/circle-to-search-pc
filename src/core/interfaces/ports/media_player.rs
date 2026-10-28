use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use crate::core::models::{AudioChunk, VideoFrame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackRequest {
    pub file_path: PathBuf,
    pub maximum_frame_width: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlaybackStatus {
    pub position: Duration,
    pub duration: Duration,
    pub is_playing: bool,
    pub has_ended: bool,
}

pub trait PlaybackSession {
    fn play(&mut self) -> Result<bool>;
    fn pause(&mut self) -> Result<bool>;
    fn seek(&mut self, position: Duration) -> Result<Duration>;
    fn status(&self) -> PlaybackStatus;
    fn take_new_frame(&mut self) -> Option<VideoFrame>;
}

pub trait MediaPlayer: Send + Sync {
    fn open(&self, request: &PlaybackRequest) -> Result<Box<dyn PlaybackSession>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimedVideoFrame {
    pub timestamp: Duration,
    pub frame: VideoFrame,
}

pub trait VideoTrackDecoder: Send {
    fn next_frame(&mut self) -> Result<Option<TimedVideoFrame>>;
}

pub trait AudioTrackDecoder: Send {
    fn next_chunk(&mut self) -> Result<Option<AudioChunk>>;
}

pub trait MediaDecoderFactory: Send + Sync {
    fn media_duration(&self, request: &PlaybackRequest) -> Result<Duration>;
    fn open_video(
        &self,
        request: &PlaybackRequest,
        start: Duration,
    ) -> Result<Box<dyn VideoTrackDecoder>>;
    fn open_audio(
        &self,
        request: &PlaybackRequest,
        start: Duration,
    ) -> Result<Option<Box<dyn AudioTrackDecoder>>>;
}

pub trait PlaybackAudioSource: Send + Sync {
    fn fill(&self, output: &mut [f32], sample_rate: u32, channel_count: u16) -> usize;
}

pub trait ActiveAudioOutput: Send {
    fn stop(self: Box<Self>) -> Result<()>;
}

pub trait AudioOutput: Send + Sync {
    fn start(&self, source: Arc<dyn PlaybackAudioSource>) -> Result<Box<dyn ActiveAudioOutput>>;
}
