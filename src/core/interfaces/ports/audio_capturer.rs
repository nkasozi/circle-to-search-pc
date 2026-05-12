use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use crate::core::models::recording::AudioSample;

pub trait AudioCapturer: Send + Sync {
    fn start_audio_capture(
        &self,
        preferred_sample_rate: Option<u32>,
    ) -> Result<Receiver<AudioSample>>;

    fn stop_audio_capture(&self) -> Result<bool>;
}
