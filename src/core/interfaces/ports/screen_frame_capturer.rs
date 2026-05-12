use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use crate::core::models::recording::{RecordingConfig, RecordingFrame};

pub trait ScreenFrameCapturer: Send + Sync {
    fn start_continuous_capture(
        &self,
        config: &RecordingConfig,
    ) -> Result<Receiver<RecordingFrame>>;

    fn stop_continuous_capture(&self) -> Result<bool>;
}
