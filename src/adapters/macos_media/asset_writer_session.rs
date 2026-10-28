mod settings;
mod writer_setup;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use objc2::rc::Retained;
use objc2_av_foundation::{
    AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVAssetWriterStatus,
};
use objc2_core_media::CMTime;

use super::foundation_helpers::wait_for_unit_completion;
use super::pixel_buffers::{create_pooled_buffer, fill_buffer_from_frame};
use crate::core::interfaces::ports::VideoEncoderSession;
use crate::core::models::{VideoEncodingRequest, VideoFrame};
use writer_setup::{
    attach_input_and_start_writing, create_pixel_buffer_adaptor, create_video_input, create_writer,
    writer_error_text,
};

const LOG_TAG: &str = "[ASSET_WRITER]";
const READY_WAIT_LIMIT: Duration = Duration::from_millis(500);
const READY_POLL_INTERVAL: Duration = Duration::from_millis(2);
const ASSET_WRITER_STARTED_EVENT: &str = "asset_writer_started";
const ASSET_WRITER_FINISHED_EVENT: &str = "asset_writer_finished";
const FRAME_SIZE_CHANGED_ERROR: &str = "frame size changed during recording";
const PIXEL_BUFFER_POOL_UNAVAILABLE_ERROR: &str = "pixel buffer pool unavailable";

pub struct AssetWriterSession {
    writer: Retained<AVAssetWriter>,
    input: Retained<AVAssetWriterInput>,
    adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
    request: VideoEncodingRequest,
    frames_written: u64,
}

unsafe impl Send for AssetWriterSession {}

fn log_writer_started(request: &VideoEncodingRequest) -> bool {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": ASSET_WRITER_STARTED_EVENT, "width": request.dimensions.width, "height": request.dimensions.height})
    );
    true
}

impl AssetWriterSession {
    pub fn start(request: &VideoEncodingRequest) -> Result<Self> {
        let _ = std::fs::remove_file(&request.output_file_path);
        let writer = create_writer(request)?;
        let input = create_video_input(request)?;
        let adaptor = create_pixel_buffer_adaptor(&input, request);
        attach_input_and_start_writing(&writer, &input, request)?;
        let _ = log_writer_started(request);
        Ok(Self {
            writer,
            input,
            adaptor,
            request: request.clone(),
            frames_written: 0,
        })
    }

    fn wait_until_ready(&self) -> Result<()> {
        let started_at = Instant::now();
        while !unsafe { self.input.isReadyForMoreMediaData() } {
            if started_at.elapsed() > READY_WAIT_LIMIT {
                bail!(
                    "video encoder is falling behind: {}",
                    writer_error_text(&self.writer)
                );
            }
            std::thread::sleep(READY_POLL_INTERVAL);
        }
        Ok(())
    }
}

impl VideoEncoderSession for AssetWriterSession {
    fn write_frame(&mut self, frame: &VideoFrame) -> Result<u64> {
        let matches_request = frame.width == self.request.dimensions.width
            && frame.height == self.request.dimensions.height;
        if !matches_request {
            bail!(FRAME_SIZE_CHANGED_ERROR);
        }
        self.wait_until_ready()?;
        let pool = unsafe { self.adaptor.pixelBufferPool() }
            .context(PIXEL_BUFFER_POOL_UNAVAILABLE_ERROR)?;
        let buffer = create_pooled_buffer(&pool)?;
        fill_buffer_from_frame(&buffer, frame)?;
        let presentation_time = unsafe {
            CMTime::new(
                self.frames_written as i64,
                self.request.frames_per_second as i32,
            )
        };
        if !unsafe {
            self.adaptor
                .appendPixelBuffer_withPresentationTime(&buffer, presentation_time)
        } {
            bail!(
                "video frame was rejected: {}",
                writer_error_text(&self.writer)
            );
        }
        self.frames_written += 1;
        Ok(self.frames_written)
    }

    fn finish(self: Box<Self>) -> Result<PathBuf> {
        unsafe { self.input.markAsFinished() };
        let writer = self.writer.clone();
        wait_for_unit_completion(|completion| unsafe {
            writer.finishWritingWithCompletionHandler(completion)
        })?;
        let status = unsafe { self.writer.status() };
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": ASSET_WRITER_FINISHED_EVENT, "frames_written": self.frames_written, "status": status.0})
        );
        if status != AVAssetWriterStatus::Completed {
            bail!(
                "video file could not be completed: {}",
                writer_error_text(&self.writer)
            );
        }
        Ok(self.request.output_file_path.clone())
    }
}
