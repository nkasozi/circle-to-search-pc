use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use super::session_state::SessionSharedState;
use crate::core::interfaces::ports::{
    ActiveCapture, AudioInputSource, ScreenFrameSource, VideoEncoder, VideoEncoderSession,
};
use crate::core::models::{FinishedRecording, ScreenCaptureRequest};

mod lifecycle;

use lifecycle::{
    build_shared_state, create_workspace, file_size_or_zero, finalize_audio_tracks, halt_capture,
    mux_final_file, remove_workspace, spawn_ticker, start_captures,
};

const LOG_TAG: &str = "[RECORDING_SESSION]";
const NO_FRAMES_CAPTURED_ERROR: &str = "No video frames were captured";
const SESSION_STARTED_EVENT: &str = "recording_session_started";
const SESSION_FINISHED_EVENT: &str = "recording_session_finished";
const SESSION_DISCARDED_EVENT: &str = "recording_session_discarded";

type TickerHandle = JoinHandle<Result<Option<Box<dyn VideoEncoderSession>>>>;

pub struct RecordingSessionPlan {
    pub capture_request: ScreenCaptureRequest,
    pub microphone_enabled: bool,
    pub working_directory: PathBuf,
    pub output_file_path: PathBuf,
}

#[derive(Clone)]
pub struct RecordingSessionDependencies {
    pub screen_source: Arc<dyn ScreenFrameSource>,
    pub microphone_source: Arc<dyn AudioInputSource>,
    pub encoder: Arc<dyn VideoEncoder>,
}

pub struct ActiveRecordingSession {
    shared: Arc<SessionSharedState>,
    captures: Vec<Box<dyn ActiveCapture>>,
    ticker: TickerHandle,
    encoder: Arc<dyn VideoEncoder>,
    output_file_path: PathBuf,
}

impl ActiveRecordingSession {
    pub fn start(
        plan: RecordingSessionPlan,
        dependencies: RecordingSessionDependencies,
    ) -> Result<Self> {
        create_workspace(&plan.working_directory)?;
        let shared = build_shared_state(&plan);
        let captures = start_captures(&shared, &dependencies, &plan)?;
        let ticker = spawn_ticker(&shared, &dependencies.encoder, &plan);
        shared.update_clock(|clock| clock.resume(Instant::now()));
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SESSION_STARTED_EVENT, "microphone": plan.microphone_enabled, "system_audio": plan.capture_request.captures_system_audio})
        );
        Ok(Self {
            shared,
            captures,
            ticker,
            encoder: dependencies.encoder,
            output_file_path: plan.output_file_path,
        })
    }

    pub fn pause(&self) -> bool {
        !self
            .shared
            .update_clock(|clock| clock.pause(Instant::now()))
    }

    pub fn resume(&self) -> bool {
        self.shared
            .update_clock(|clock| clock.resume(Instant::now()))
    }

    pub fn is_paused(&self) -> bool {
        self.shared
            .clock_snapshot()
            .is_some_and(|clock| !clock.is_running())
    }

    pub fn elapsed(&self) -> Duration {
        self.shared.active_elapsed(Instant::now())
    }

    pub fn failure(&self) -> Option<String> {
        self.shared.failure()
    }

    pub fn stop(self) -> Result<FinishedRecording> {
        let duration = self.elapsed();
        let Self {
            shared,
            captures,
            ticker,
            encoder,
            output_file_path,
        } = self;
        let encoder_session =
            halt_capture(&shared, captures, ticker)?.context(NO_FRAMES_CAPTURED_ERROR)?;
        let video_file_path = encoder_session.finish()?;
        let audio_file_paths = finalize_audio_tracks(&shared);
        let final_path = mux_final_file(
            &encoder,
            video_file_path,
            audio_file_paths,
            &output_file_path,
        )?;
        let _ = remove_workspace(&shared.working_directory);
        let size_bytes = file_size_or_zero(&final_path);
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SESSION_FINISHED_EVENT, "path": final_path, "duration_seconds": duration.as_secs_f64(), "size_bytes": size_bytes})
        );
        Ok(FinishedRecording {
            file_path: final_path,
            duration,
            size_bytes,
        })
    }

    pub fn discard(self) -> Result<bool> {
        let Self {
            shared,
            captures,
            ticker,
            ..
        } = self;
        if let Some(session) = halt_capture(&shared, captures, ticker)? {
            let _ = session.finish();
        }
        let _ = shared.take_audio_writers();
        let _ = remove_workspace(&shared.working_directory);
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SESSION_DISCARDED_EVENT})
        );
        Ok(true)
    }
}
