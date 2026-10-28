use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};

use super::{RecordingSessionDependencies, RecordingSessionPlan, TickerHandle, LOG_TAG};
use crate::core::interfaces::ports::{ActiveCapture, MediaSink, VideoEncoder, VideoEncoderSession};
use crate::core::models::{AudioMuxRequest, AudioTrackKind};
use crate::core::recording::frame_ticker::{run_frame_ticker, FrameTickerPlan};
use crate::core::recording::session_state::SessionSharedState;

const VIDEO_TRACK_FILE_NAME: &str = "video.mp4";
const TICKER_THREAD_PANICKED_ERROR: &str = "frame ticker thread panicked";
const WORKSPACE_CREATION_ERROR: &str = "failed to create recording workspace";
const RECORDINGS_FOLDER_CREATION_ERROR: &str = "failed to create recordings folder";
const CAPTURE_STOP_FAILED_EVENT: &str = "capture_stop_failed";
const AUDIO_TRACK_FINALIZE_FAILED_EVENT: &str = "audio_track_finalize_failed";

pub(super) fn create_workspace(working_directory: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(working_directory).context(WORKSPACE_CREATION_ERROR)?;
    Ok(working_directory.to_path_buf())
}

pub(super) fn build_shared_state(plan: &RecordingSessionPlan) -> Arc<SessionSharedState> {
    Arc::new(SessionSharedState::build(
        plan.working_directory.clone(),
        enabled_audio_tracks(plan),
    ))
}

pub(super) fn start_captures(
    shared: &Arc<SessionSharedState>,
    dependencies: &RecordingSessionDependencies,
    plan: &RecordingSessionPlan,
) -> Result<Vec<Box<dyn ActiveCapture>>> {
    let sink: Arc<dyn MediaSink> = shared.clone();
    let mut captures = vec![dependencies
        .screen_source
        .start_capture(plan.capture_request.clone(), sink.clone())?];
    if plan.microphone_enabled {
        captures.push(dependencies.microphone_source.start_capture(sink)?);
    }
    Ok(captures)
}

fn enabled_audio_tracks(plan: &RecordingSessionPlan) -> Vec<AudioTrackKind> {
    let microphone = plan
        .microphone_enabled
        .then_some(AudioTrackKind::Microphone);
    let system_audio = plan
        .capture_request
        .captures_system_audio
        .then_some(AudioTrackKind::SystemAudio);
    microphone.into_iter().chain(system_audio).collect()
}

pub(super) fn spawn_ticker(
    shared: &Arc<SessionSharedState>,
    encoder: &Arc<dyn VideoEncoder>,
    plan: &RecordingSessionPlan,
) -> TickerHandle {
    let ticker_shared = Arc::clone(shared);
    let ticker_encoder = Arc::clone(encoder);
    let ticker_plan = FrameTickerPlan {
        frames_per_second: plan.capture_request.frames_per_second,
        video_file_path: plan.working_directory.join(VIDEO_TRACK_FILE_NAME),
    };
    std::thread::spawn(move || {
        let outcome = run_frame_ticker(Arc::clone(&ticker_shared), ticker_encoder, ticker_plan);
        if let Err(ticker_error) = &outcome {
            ticker_shared.record_failure(ticker_error.to_string());
        }
        outcome
    })
}

pub(super) fn halt_capture(
    shared: &Arc<SessionSharedState>,
    captures: Vec<Box<dyn ActiveCapture>>,
    ticker: TickerHandle,
) -> Result<Option<Box<dyn VideoEncoderSession>>> {
    let _ = stop_captures(captures);
    shared.request_stop();
    ticker
        .join()
        .map_err(|_| anyhow::anyhow!(TICKER_THREAD_PANICKED_ERROR))?
}

fn stop_captures(captures: Vec<Box<dyn ActiveCapture>>) -> usize {
    let mut failed_stop_count = 0usize;
    for capture in captures {
        let Err(stop_error) = capture.stop() else {
            continue;
        };
        log::warn!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CAPTURE_STOP_FAILED_EVENT, "error": stop_error.to_string()})
        );
        failed_stop_count += 1;
    }
    failed_stop_count
}

pub(super) fn finalize_audio_tracks(shared: &SessionSharedState) -> Vec<PathBuf> {
    shared
        .take_audio_writers()
        .into_iter()
        .filter(|(_, writer)| writer.has_audio())
        .filter_map(|(track, writer)| match writer.finalize() {
            Ok(path) => Some(path),
            Err(finalize_error) => {
                log::warn!("{} {}", LOG_TAG, serde_json::json!({"event": AUDIO_TRACK_FINALIZE_FAILED_EVENT, "track": format!("{:?}", track), "error": finalize_error.to_string()}));
                None
            }
        })
        .collect()
}

pub(super) fn mux_final_file(
    encoder: &Arc<dyn VideoEncoder>,
    video_file_path: PathBuf,
    audio_file_paths: Vec<PathBuf>,
    output_file_path: &PathBuf,
) -> Result<PathBuf> {
    if let Some(parent_directory) = output_file_path.parent() {
        std::fs::create_dir_all(parent_directory).context(RECORDINGS_FOLDER_CREATION_ERROR)?;
    }
    encoder.mux_audio(&AudioMuxRequest {
        video_file_path,
        audio_file_paths,
        output_file_path: output_file_path.clone(),
    })
}

pub(super) fn file_size_or_zero(file_path: &Path) -> u64 {
    std::fs::metadata(file_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

pub(super) fn remove_workspace(working_directory: &Path) -> bool {
    std::fs::remove_dir_all(working_directory).is_ok()
}
