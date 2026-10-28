use std::path::{Path, PathBuf};

use anyhow::Result;

use super::asset_muxer::mux_recording;
use super::asset_writer_session::AssetWriterSession;
use super::thumbnail::write_thumbnail;
use crate::core::interfaces::ports::{EncoderAvailability, VideoEncoder, VideoEncoderSession};
use crate::core::models::{AudioMuxRequest, VideoEncodingRequest};

const NATIVE_ENCODER_NAME: &str = "AVFoundation H.264";

pub struct NativeVideoEncoder;

impl VideoEncoder for NativeVideoEncoder {
    fn availability(&self) -> EncoderAvailability {
        EncoderAvailability::Ready {
            encoder_name: NATIVE_ENCODER_NAME.to_string(),
        }
    }

    fn start_session(
        &self,
        request: &VideoEncodingRequest,
    ) -> Result<Box<dyn VideoEncoderSession>> {
        Ok(Box::new(AssetWriterSession::start(request)?))
    }

    fn mux_audio(&self, request: &AudioMuxRequest) -> Result<PathBuf> {
        mux_recording(request)
    }

    fn extract_thumbnail(&self, video_file_path: &Path, thumbnail_path: &Path) -> Result<PathBuf> {
        write_thumbnail(video_file_path, thumbnail_path)
    }
}
