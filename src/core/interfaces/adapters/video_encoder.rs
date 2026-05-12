use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::core::models::recording::{AudioSample, EncodingConfig, RecordingFrame};

#[async_trait]
pub trait VideoEncoder: Send + Sync {
    async fn start_encoding(&mut self, config: EncodingConfig) -> Result<bool>;

    async fn write_video_frame(&mut self, frame: &RecordingFrame) -> Result<bool>;

    async fn write_audio_samples(&mut self, sample: &AudioSample) -> Result<bool>;

    async fn finish_encoding(&mut self) -> Result<PathBuf>;
}
