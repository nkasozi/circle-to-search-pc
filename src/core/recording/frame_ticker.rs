use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;

use super::recording_clock::frames_due;
use super::session_state::SessionSharedState;
use crate::core::interfaces::ports::{VideoEncoder, VideoEncoderSession};
use crate::core::models::{OutputDimensions, VideoEncodingRequest, VideoFrame};

const TICKER_SLEEP_MILLISECONDS: u64 = 4;
const LOG_TAG: &str = "[FRAME_TICKER]";
const FRAME_TICKER_STOPPED_EVENT: &str = "frame_ticker_stopped";

pub struct FrameTickerPlan {
    pub frames_per_second: u32,
    pub video_file_path: std::path::PathBuf,
}

pub fn run_frame_ticker(
    shared: Arc<SessionSharedState>,
    encoder: Arc<dyn VideoEncoder>,
    plan: FrameTickerPlan,
) -> Result<Option<Box<dyn VideoEncoderSession>>> {
    let mut encoder_session: Option<Box<dyn VideoEncoderSession>> = None;
    let mut frames_written: u64 = 0;
    while !shared.is_stop_requested() {
        std::thread::sleep(Duration::from_millis(TICKER_SLEEP_MILLISECONDS));
        let Some(clock) = shared.clock_snapshot() else {
            continue;
        };
        if !clock.is_running() {
            continue;
        }
        let due = frames_due(clock.active_elapsed(Instant::now()), plan.frames_per_second);
        let Some(frame) = shared.latest_frame() else {
            continue;
        };
        let session = match encoder_session.as_mut() {
            Some(session) => session,
            None => encoder_session.insert(start_encoder(&encoder, &frame, &plan)?),
        };
        frames_written = write_due_frames(session.as_mut(), &frame, frames_written, due)?;
    }
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": FRAME_TICKER_STOPPED_EVENT, "frames_written": frames_written})
    );
    Ok(encoder_session)
}

fn start_encoder(
    encoder: &Arc<dyn VideoEncoder>,
    first_frame: &VideoFrame,
    plan: &FrameTickerPlan,
) -> Result<Box<dyn VideoEncoderSession>> {
    let request = VideoEncodingRequest {
        output_file_path: plan.video_file_path.clone(),
        dimensions: OutputDimensions {
            width: first_frame.width,
            height: first_frame.height,
        },
        pixel_layout: first_frame.pixel_layout,
        frames_per_second: plan.frames_per_second,
    };
    encoder.start_session(&request)
}

fn write_due_frames(
    session: &mut dyn VideoEncoderSession,
    frame: &VideoFrame,
    frames_already_written: u64,
    frames_due_now: u64,
) -> Result<u64> {
    let mut frames_written = frames_already_written;
    while frames_written < frames_due_now {
        session.write_frame(frame)?;
        frames_written += 1;
    }
    Ok(frames_written)
}
