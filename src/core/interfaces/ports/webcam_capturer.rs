use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use crate::core::models::recording::RecordingFrame;

pub trait WebcamCapturer: Send + Sync {
    fn start_webcam_capture(
        &self,
        requested_width: u32,
        requested_height: u32,
    ) -> Result<Receiver<RecordingFrame>>;

    fn stop_webcam_capture(&self) -> Result<bool>;
}
