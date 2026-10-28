mod audio_track;
pub mod frame_reader;
mod media_decoder;
mod muxer;
mod runtime;
mod sink_writer_session;
mod thumbnail;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub use media_decoder::MediaFoundationDecoderFactory;

use crate::core::interfaces::ports::{EncoderAvailability, VideoEncoder, VideoEncoderSession};
use crate::core::models::{AudioMuxRequest, VideoEncodingRequest};

const MEDIA_FOUNDATION_ENCODER_NAME: &str = "Media Foundation H.264";

pub struct MediaFoundationVideoEncoder;

impl VideoEncoder for MediaFoundationVideoEncoder {
    fn availability(&self) -> EncoderAvailability {
        EncoderAvailability::Ready {
            encoder_name: MEDIA_FOUNDATION_ENCODER_NAME.to_string(),
        }
    }

    fn start_session(
        &self,
        request: &VideoEncodingRequest,
    ) -> Result<Box<dyn VideoEncoderSession>> {
        Ok(Box::new(sink_writer_session::SinkWriterSession::start(
            request,
        )?))
    }

    fn mux_audio(&self, request: &AudioMuxRequest) -> Result<PathBuf> {
        muxer::mux_recording(request)
    }

    fn extract_thumbnail(&self, video_file_path: &Path, thumbnail_path: &Path) -> Result<PathBuf> {
        thumbnail::write_thumbnail(video_file_path, thumbnail_path)
    }
}
