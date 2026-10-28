use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::core::models::{AudioMuxRequest, VideoEncodingRequest, VideoFrame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncoderAvailability {
    Ready { encoder_name: String },
    Missing { reason: String },
}

pub trait VideoEncoderSession: Send {
    fn write_frame(&mut self, frame: &VideoFrame) -> Result<u64>;
    fn finish(self: Box<Self>) -> Result<PathBuf>;
}

pub trait VideoEncoder: Send + Sync {
    fn availability(&self) -> EncoderAvailability;
    fn start_session(&self, request: &VideoEncodingRequest)
        -> Result<Box<dyn VideoEncoderSession>>;
    fn mux_audio(&self, request: &AudioMuxRequest) -> Result<PathBuf>;
    fn extract_thumbnail(&self, video_file_path: &Path, thumbnail_path: &Path) -> Result<PathBuf>;
}
