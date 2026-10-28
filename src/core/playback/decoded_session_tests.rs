use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::decoded_session::{DecodedPlaybackDependencies, DecodedPlaybackSession};
use crate::core::interfaces::ports::{
    ActiveAudioOutput, AudioOutput, AudioTrackDecoder, MediaDecoderFactory, PlaybackAudioSource,
    PlaybackRequest, PlaybackSession, TimedVideoFrame, VideoTrackDecoder,
};
use crate::core::models::{AudioChunk, PixelLayout, VideoFrame};

const FRAMES_PER_SECOND: u64 = 30;
const MEDIA_DURATION: Duration = Duration::from_millis(1_000);

struct SyntheticVideo {
    next_index: u64,
}

impl VideoTrackDecoder for SyntheticVideo {
    fn next_frame(&mut self) -> anyhow::Result<Option<TimedVideoFrame>> {
        let timestamp = Duration::from_millis(self.next_index * 1_000 / FRAMES_PER_SECOND);
        if timestamp >= MEDIA_DURATION {
            return Ok(None);
        }
        self.next_index += 1;
        let frame = VideoFrame {
            width: 2,
            height: 2,
            pixel_layout: PixelLayout::Rgba,
            data: Arc::new(vec![self.next_index as u8; 16]),
        };
        Ok(Some(TimedVideoFrame { timestamp, frame }))
    }
}

struct SyntheticAudio {
    chunks_left: usize,
}

impl AudioTrackDecoder for SyntheticAudio {
    fn next_chunk(&mut self) -> anyhow::Result<Option<AudioChunk>> {
        if self.chunks_left == 0 {
            return Ok(None);
        }
        self.chunks_left -= 1;
        Ok(Some(AudioChunk {
            sample_rate: 24_000,
            channel_count: 1,
            interleaved_samples: vec![0.1; 2_400],
        }))
    }
}

struct SyntheticFactory;

impl MediaDecoderFactory for SyntheticFactory {
    fn media_duration(&self, _request: &PlaybackRequest) -> anyhow::Result<Duration> {
        Ok(MEDIA_DURATION)
    }

    fn open_video(
        &self,
        _request: &PlaybackRequest,
        start: Duration,
    ) -> anyhow::Result<Box<dyn VideoTrackDecoder>> {
        let first_index = start.as_millis() as u64 * FRAMES_PER_SECOND / 1_000;
        Ok(Box::new(SyntheticVideo {
            next_index: first_index,
        }))
    }

    fn open_audio(
        &self,
        _request: &PlaybackRequest,
        _start: Duration,
    ) -> anyhow::Result<Option<Box<dyn AudioTrackDecoder>>> {
        Ok(Some(Box::new(SyntheticAudio { chunks_left: 10 })))
    }
}

struct SyntheticOutput {
    filled_samples: Arc<AtomicUsize>,
}

struct SyntheticActiveOutput {
    running: Arc<AtomicBool>,
}

impl ActiveAudioOutput for SyntheticActiveOutput {
    fn stop(self: Box<Self>) -> anyhow::Result<()> {
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }
}

impl AudioOutput for SyntheticOutput {
    fn start(
        &self,
        source: Arc<dyn PlaybackAudioSource>,
    ) -> anyhow::Result<Box<dyn ActiveAudioOutput>> {
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let filled_samples = Arc::clone(&self.filled_samples);
        std::thread::spawn(move || {
            let mut buffer = vec![0.0f32; 960];
            while worker_running.load(Ordering::SeqCst) {
                filled_samples.fetch_add(source.fill(&mut buffer, 48_000, 2), Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(10));
            }
        });
        Ok(Box::new(SyntheticActiveOutput { running }))
    }
}

fn open_session(filled_samples: Arc<AtomicUsize>) -> DecodedPlaybackSession {
    let request = PlaybackRequest {
        file_path: "synthetic.mp4".into(),
        maximum_frame_width: 640,
    };
    DecodedPlaybackSession::open(
        &request,
        DecodedPlaybackDependencies {
            decoder_factory: Arc::new(SyntheticFactory),
            audio_output: Arc::new(SyntheticOutput { filled_samples }),
        },
    )
    .expect("session opens")
}

#[test]
fn test_session_starts_paused_and_shows_first_frame() {
    let mut session = open_session(Arc::new(AtomicUsize::new(0)));
    std::thread::sleep(Duration::from_millis(50));
    let status = session.status();
    assert!(!status.is_playing);
    assert_eq!(status.duration, MEDIA_DURATION);
    assert!(session.take_new_frame().is_some());
}

#[test]
fn test_playing_advances_frames_and_feeds_audio() {
    let filled_samples = Arc::new(AtomicUsize::new(0));
    let mut session = open_session(Arc::clone(&filled_samples));
    assert!(session.play().unwrap_or(false));
    std::thread::sleep(Duration::from_millis(300));
    let frame = session.take_new_frame().expect("frame while playing");
    assert!(
        frame.data[0] >= 4,
        "expected a later frame, got index {}",
        frame.data[0]
    );
    assert!(session.status().position >= Duration::from_millis(250));
    assert!(filled_samples.load(Ordering::SeqCst) > 0);
}

#[test]
fn test_seek_jumps_to_later_frames() {
    let mut session = open_session(Arc::new(AtomicUsize::new(0)));
    let target = session.seek(Duration::from_millis(800)).expect("seek");
    assert_eq!(target, Duration::from_millis(800));
    std::thread::sleep(Duration::from_millis(50));
    let frame = session.take_new_frame().expect("frame after seek");
    assert!(frame.data[0] >= 24);
    assert_eq!(session.status().position, Duration::from_millis(800));
}

#[test]
fn test_playback_stops_at_the_end_and_replays_from_start() {
    let mut session = open_session(Arc::new(AtomicUsize::new(0)));
    session
        .seek(Duration::from_millis(950))
        .expect("seek near end");
    session.play().expect("play");
    std::thread::sleep(Duration::from_millis(150));
    let _ = session.take_new_frame();
    let status = session.status();
    assert!(status.has_ended);
    assert!(!status.is_playing);
    session.play().expect("replay");
    assert!(session.status().position < Duration::from_millis(100));
}
