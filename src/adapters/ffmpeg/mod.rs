#![cfg_attr(any(target_os = "macos", target_os = "windows"), allow(dead_code))]

mod arguments;
mod flags;
mod locator;
mod media_probe;
mod playback_arguments;
mod playback_decoders;
mod process;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin};
use std::thread::JoinHandle;

use anyhow::{bail, Context, Result};

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use playback_decoders::FfmpegDecoderFactory;

use crate::core::interfaces::ports::{EncoderAvailability, VideoEncoder, VideoEncoderSession};
use crate::core::models::{AudioMuxRequest, VideoEncodingRequest, VideoFrame};

const LOG_TAG: &str = "[FFMPEG]";
const FRAME_SIZE_MISMATCH_ERROR: &str = "frame does not match the encoder dimensions";
const ENCODER_INPUT_CLOSED_ERROR: &str = "encoder input is already closed";
const FFMPEG_MISSING_ERROR: &str = locator::BUNDLED_ENCODER_MISSING_MESSAGE;
const SEND_FRAME_FAILED_ERROR: &str = "failed to send frame to ffmpeg";
const WAIT_FOR_FFMPEG_FAILED_ERROR: &str = "failed waiting for ffmpeg";
const ENCODER_SESSION_STARTED_EVENT: &str = "encoder_session_started";
const ENCODER_SESSION_FINISHED_EVENT: &str = "encoder_session_finished";

pub struct FfmpegVideoEncoder;

impl FfmpegVideoEncoder {
    fn require_executable(&self) -> Result<PathBuf> {
        locator::locate_bundled_ffmpeg()
            .filter(|path| process::is_runnable(path))
            .context(FFMPEG_MISSING_ERROR)
    }
}

impl VideoEncoder for FfmpegVideoEncoder {
    fn availability(&self) -> EncoderAvailability {
        match locator::locate_bundled_ffmpeg().filter(|path| process::is_runnable(path)) {
            Some(executable_path) => EncoderAvailability::Ready {
                encoder_name: executable_path.to_string_lossy().to_string(),
            },
            None => EncoderAvailability::Missing {
                reason: locator::BUNDLED_ENCODER_MISSING_MESSAGE.to_string(),
            },
        }
    }

    fn start_session(
        &self,
        request: &VideoEncodingRequest,
    ) -> Result<Box<dyn VideoEncoderSession>> {
        let executable_path = self.require_executable()?;
        let encode_arguments =
            arguments::build_encode_arguments(request, arguments::platform_video_codec());
        let mut child = process::spawn_with_piped_input(&executable_path, &encode_arguments)?;
        let stdin = child.stdin.take().context(ENCODER_INPUT_CLOSED_ERROR)?;
        let stderr_collector = process::collect_stderr_in_background(child.stderr.take());
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": ENCODER_SESSION_STARTED_EVENT, "width": request.dimensions.width, "height": request.dimensions.height, "fps": request.frames_per_second})
        );
        Ok(Box::new(FfmpegEncoderSession {
            child,
            stdin: Some(stdin),
            stderr_collector,
            request: request.clone(),
            frames_written: 0,
        }))
    }

    fn mux_audio(&self, request: &AudioMuxRequest) -> Result<PathBuf> {
        let executable_path = self.require_executable()?;
        process::run_to_completion(&executable_path, &arguments::build_mux_arguments(request))?;
        Ok(request.output_file_path.clone())
    }

    fn extract_thumbnail(&self, video_file_path: &Path, thumbnail_path: &Path) -> Result<PathBuf> {
        let executable_path = self.require_executable()?;
        let thumbnail_arguments =
            arguments::build_thumbnail_arguments(video_file_path, thumbnail_path);
        process::run_to_completion(&executable_path, &thumbnail_arguments)?;
        Ok(thumbnail_path.to_path_buf())
    }
}

struct FfmpegEncoderSession {
    child: Child,
    stdin: Option<ChildStdin>,
    stderr_collector: Option<JoinHandle<String>>,
    request: VideoEncodingRequest,
    frames_written: u64,
}

impl VideoEncoderSession for FfmpegEncoderSession {
    fn write_frame(&mut self, frame: &VideoFrame) -> Result<u64> {
        let matches_request = frame.width == self.request.dimensions.width
            && frame.height == self.request.dimensions.height
            && frame.pixel_layout == self.request.pixel_layout;
        if !matches_request {
            bail!("{}: {:?}", FRAME_SIZE_MISMATCH_ERROR, frame);
        }
        let stdin = self.stdin.as_mut().context(ENCODER_INPUT_CLOSED_ERROR)?;
        stdin
            .write_all(&frame.data)
            .context(SEND_FRAME_FAILED_ERROR)?;
        self.frames_written += 1;
        Ok(self.frames_written)
    }

    fn finish(mut self: Box<Self>) -> Result<PathBuf> {
        drop(self.stdin.take());
        let exit_status = self.child.wait().context(WAIT_FOR_FFMPEG_FAILED_ERROR)?;
        let error_output = self
            .stderr_collector
            .take()
            .and_then(|collector| collector.join().ok())
            .unwrap_or_default();
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": ENCODER_SESSION_FINISHED_EVENT, "frames_written": self.frames_written, "status": exit_status.code()})
        );
        if !exit_status.success() {
            bail!(
                "ffmpeg encoder failed: {}",
                process::truncate_error(&error_output)
            );
        }
        Ok(self.request.output_file_path.clone())
    }
}
