use std::sync::Arc;

use anyhow::Result;

use crate::core::models::{AudioChunk, AudioTrackKind, ScreenCaptureRequest, VideoFrame};

pub trait MediaSink: Send + Sync {
    fn accept_video_frame(&self, frame: VideoFrame) -> bool;
    fn accept_audio_chunk(&self, track: AudioTrackKind, chunk: AudioChunk) -> bool;
}

pub trait ActiveCapture: Send {
    fn stop(self: Box<Self>) -> Result<()>;
}

pub trait ScreenFrameSource: Send + Sync {
    fn start_capture(
        &self,
        request: ScreenCaptureRequest,
        sink: Arc<dyn MediaSink>,
    ) -> Result<Box<dyn ActiveCapture>>;
}

pub trait AudioInputSource: Send + Sync {
    fn start_capture(&self, sink: Arc<dyn MediaSink>) -> Result<Box<dyn ActiveCapture>>;
}

pub trait CameraSource: Send + Sync {
    fn start_capture(&self, sink: Arc<dyn MediaSink>) -> Result<Box<dyn ActiveCapture>>;
}
