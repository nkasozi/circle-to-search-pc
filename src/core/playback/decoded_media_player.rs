use std::sync::Arc;

use anyhow::Result;

use super::decoded_session::{DecodedPlaybackDependencies, DecodedPlaybackSession};
use crate::core::interfaces::ports::{
    AudioOutput, MediaDecoderFactory, MediaPlayer, PlaybackRequest, PlaybackSession,
};

pub struct DecodedMediaPlayer {
    pub decoder_factory: Arc<dyn MediaDecoderFactory>,
    pub audio_output: Arc<dyn AudioOutput>,
}

impl MediaPlayer for DecodedMediaPlayer {
    fn open(&self, request: &PlaybackRequest) -> Result<Box<dyn PlaybackSession>> {
        let dependencies = DecodedPlaybackDependencies {
            decoder_factory: Arc::clone(&self.decoder_factory),
            audio_output: Arc::clone(&self.audio_output),
        };
        Ok(Box::new(DecodedPlaybackSession::open(
            request,
            dependencies,
        )?))
    }
}
