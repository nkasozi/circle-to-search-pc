use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::recording_clock::RecordingClock;
use super::wav_writer::WavFileWriter;
use crate::core::interfaces::ports::MediaSink;
use crate::core::models::{AudioChunk, AudioTrackKind, VideoFrame};

const LOG_TAG: &str = "[RECORDING_SESSION]";
const MICROPHONE_FILE_NAME: &str = "microphone.wav";
const SYSTEM_AUDIO_FILE_NAME: &str = "system-audio.wav";
const AUDIO_WRITER_LOCK_POISONED_ERROR: &str = "audio writer lock poisoned";
const AUDIO_FORMAT_CHANGED_ERROR: &str = "audio format changed mid-recording";
const RECORDING_FAILURE_EVENT: &str = "recording_failure";
const AUDIO_CHUNK_DROPPED_EVENT: &str = "audio_chunk_dropped";

pub struct SessionSharedState {
    pub working_directory: PathBuf,
    pub enabled_audio_tracks: Vec<AudioTrackKind>,
    latest_frame: Mutex<Option<VideoFrame>>,
    clock: Mutex<RecordingClock>,
    audio_writers: Mutex<Vec<(AudioTrackKind, WavFileWriter)>>,
    stop_requested: AtomicBool,
    failure: Mutex<Option<String>>,
}

impl SessionSharedState {
    pub fn build(working_directory: PathBuf, enabled_audio_tracks: Vec<AudioTrackKind>) -> Self {
        Self {
            working_directory,
            enabled_audio_tracks,
            latest_frame: Mutex::new(None),
            clock: Mutex::new(RecordingClock::Paused {
                accumulated: Duration::ZERO,
            }),
            audio_writers: Mutex::new(Vec::new()),
            stop_requested: AtomicBool::new(false),
            failure: Mutex::new(None),
        }
    }

    pub fn latest_frame(&self) -> Option<VideoFrame> {
        self.latest_frame.lock().ok().and_then(|slot| slot.clone())
    }

    pub fn clock_snapshot(&self) -> Option<RecordingClock> {
        self.clock.lock().ok().map(|clock| *clock)
    }

    pub fn update_clock(&self, transition: impl FnOnce(RecordingClock) -> RecordingClock) -> bool {
        self.clock
            .lock()
            .map(|mut clock| {
                *clock = transition(*clock);
                clock.is_running()
            })
            .unwrap_or(false)
    }

    pub fn active_elapsed(&self, now: Instant) -> Duration {
        self.clock_snapshot()
            .map(|clock| clock.active_elapsed(now))
            .unwrap_or(Duration::ZERO)
    }

    pub fn request_stop(&self) -> bool {
        !self.stop_requested.swap(true, Ordering::SeqCst)
    }

    pub fn is_stop_requested(&self) -> bool {
        self.stop_requested.load(Ordering::SeqCst)
    }

    pub fn record_failure(&self, message: String) -> bool {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": RECORDING_FAILURE_EVENT, "error": message})
        );
        self.failure
            .lock()
            .map(|mut failure| {
                let is_first_failure = failure.is_none();
                failure.get_or_insert(message);
                is_first_failure
            })
            .unwrap_or(false)
    }

    pub fn failure(&self) -> Option<String> {
        self.failure.lock().ok().and_then(|failure| failure.clone())
    }

    pub fn take_audio_writers(&self) -> Vec<(AudioTrackKind, WavFileWriter)> {
        self.audio_writers
            .lock()
            .map(|mut writers| std::mem::take(&mut *writers))
            .unwrap_or_default()
    }

    fn append_audio(&self, track: AudioTrackKind, chunk: &AudioChunk) -> anyhow::Result<u32> {
        let mut writers = self
            .audio_writers
            .lock()
            .map_err(|_| anyhow::anyhow!(AUDIO_WRITER_LOCK_POISONED_ERROR))?;
        let existing_index = writers.iter().position(|(kind, _)| *kind == track);
        let writer_index = match existing_index {
            Some(index) => index,
            None => {
                let file_path = self.working_directory.join(audio_file_name(track));
                writers.push((
                    track,
                    WavFileWriter::create(&file_path, chunk.sample_rate, chunk.channel_count)?,
                ));
                writers.len() - 1
            }
        };
        let writer = &mut writers[writer_index].1;
        if !writer.format_matches(chunk.sample_rate, chunk.channel_count) {
            anyhow::bail!(AUDIO_FORMAT_CHANGED_ERROR);
        }
        writer.append_samples(&chunk.interleaved_samples)
    }
}

fn audio_file_name(track: AudioTrackKind) -> &'static str {
    match track {
        AudioTrackKind::Microphone => MICROPHONE_FILE_NAME,
        AudioTrackKind::SystemAudio => SYSTEM_AUDIO_FILE_NAME,
    }
}

impl MediaSink for SessionSharedState {
    fn accept_video_frame(&self, frame: VideoFrame) -> bool {
        self.latest_frame
            .lock()
            .map(|mut slot| slot.replace(frame).is_none())
            .unwrap_or(false)
    }

    fn accept_audio_chunk(&self, track: AudioTrackKind, chunk: AudioChunk) -> bool {
        let is_running = self
            .clock_snapshot()
            .is_some_and(|clock| clock.is_running());
        if !is_running || !self.enabled_audio_tracks.contains(&track) {
            return false;
        }
        match self.append_audio(track, &chunk) {
            Ok(_) => true,
            Err(append_error) => {
                log::warn!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": AUDIO_CHUNK_DROPPED_EVENT, "track": format!("{:?}", track), "error": append_error.to_string()})
                );
                false
            }
        }
    }
}
