use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;

use super::audio_ring::AudioRing;
use super::decoder_pumps::{AudioPump, VideoPump};
use super::frame_selection::take_due_frame;
use super::playback_clock::PlaybackClock;
use crate::core::interfaces::ports::{
    ActiveAudioOutput, AudioOutput, MediaDecoderFactory, PlaybackAudioSource, PlaybackRequest,
    PlaybackSession, PlaybackStatus, TimedVideoFrame,
};
use crate::core::models::VideoFrame;

const LOG_TAG: &str = "[PLAYBACK]";
const MAXIMUM_PENDING_FRAMES: usize = 12;
const AUDIO_OUTPUT_UNAVAILABLE_EVENT: &str = "audio_output_unavailable";
const PLAYBACK_OPENED_EVENT: &str = "playback_opened";

pub struct DecodedPlaybackDependencies {
    pub decoder_factory: Arc<dyn MediaDecoderFactory>,
    pub audio_output: Arc<dyn AudioOutput>,
}

pub struct DecodedPlaybackSession {
    dependencies: DecodedPlaybackDependencies,
    request: PlaybackRequest,
    duration: Duration,
    clock: PlaybackClock,
    video_pump: Option<VideoPump>,
    audio_pump: Option<AudioPump>,
    pending_frames: VecDeque<TimedVideoFrame>,
    audio_ring: Arc<AudioRing>,
    audio_output: Option<Box<dyn ActiveAudioOutput>>,
}

impl DecodedPlaybackSession {
    pub fn open(
        request: &PlaybackRequest,
        dependencies: DecodedPlaybackDependencies,
    ) -> Result<Self> {
        let duration = dependencies.decoder_factory.media_duration(request)?;
        let audio_ring = Arc::new(AudioRing::default());
        let source: Arc<dyn PlaybackAudioSource> = audio_ring.clone();
        let audio_output = match dependencies.audio_output.start(source) {
            Ok(output) => Some(output),
            Err(output_error) => {
                log::warn!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": AUDIO_OUTPUT_UNAVAILABLE_EVENT, "error": output_error.to_string()})
                );
                None
            }
        };
        let mut session = Self {
            dependencies,
            request: request.clone(),
            duration,
            clock: PlaybackClock::paused_at(Duration::ZERO),
            video_pump: None,
            audio_pump: None,
            pending_frames: VecDeque::new(),
            audio_ring,
            audio_output,
        };
        session.restart_decoders(Duration::ZERO)?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": PLAYBACK_OPENED_EVENT, "duration_seconds": duration.as_secs_f64()})
        );
        Ok(session)
    }

    fn restart_decoders(&mut self, position: Duration) -> Result<()> {
        self.video_pump = None;
        self.audio_pump = None;
        self.pending_frames.clear();
        self.audio_ring.clear();
        let factory = Arc::clone(&self.dependencies.decoder_factory);
        self.video_pump = Some(VideoPump::start(
            factory.open_video(&self.request, position)?,
        ));
        self.audio_pump = factory
            .open_audio(&self.request, position)?
            .map(|decoder| AudioPump::start(decoder, Arc::clone(&self.audio_ring)));
        Ok(())
    }

    fn current_position(&self) -> Duration {
        self.clock.position(Instant::now(), self.duration)
    }

    fn collect_decoded_frames(&mut self) -> usize {
        let Some(pump) = self.video_pump.as_ref() else {
            return 0;
        };
        let mut collected = 0usize;
        while self.pending_frames.len() < MAXIMUM_PENDING_FRAMES {
            match pump.frames.try_recv() {
                Ok(frame) => {
                    self.pending_frames.push_back(frame);
                    collected += 1;
                }
                Err(_) => break,
            }
        }
        collected
    }
}

impl PlaybackSession for DecodedPlaybackSession {
    fn play(&mut self) -> Result<bool> {
        if self.current_position() >= self.duration {
            self.seek(Duration::ZERO)?;
        }
        self.clock = self.clock.play(Instant::now(), self.duration);
        self.audio_ring.set_playing(true);
        Ok(true)
    }

    fn pause(&mut self) -> Result<bool> {
        self.clock = self.clock.pause(Instant::now(), self.duration);
        self.audio_ring.set_playing(false);
        Ok(false)
    }

    fn seek(&mut self, position: Duration) -> Result<Duration> {
        let target = position.min(self.duration);
        let was_playing = self.clock.is_playing();
        self.restart_decoders(target)?;
        self.clock = PlaybackClock::paused_at(target);
        if was_playing {
            self.clock = self.clock.play(Instant::now(), self.duration);
        }
        Ok(target)
    }

    fn status(&self) -> PlaybackStatus {
        let position = self.current_position();
        PlaybackStatus {
            position,
            duration: self.duration,
            is_playing: self.clock.is_playing() && position < self.duration,
            has_ended: position >= self.duration,
        }
    }

    fn take_new_frame(&mut self) -> Option<VideoFrame> {
        self.collect_decoded_frames();
        let position = self.current_position();
        if position >= self.duration && self.clock.is_playing() {
            let _ = self.pause();
        }
        let due = take_due_frame(&mut self.pending_frames, position);
        due.map(|timed| timed.frame)
    }
}

impl Drop for DecodedPlaybackSession {
    fn drop(&mut self) {
        self.video_pump = None;
        self.audio_pump = None;
        if let Some(output) = self.audio_output.take() {
            let _ = output.stop();
        }
    }
}
