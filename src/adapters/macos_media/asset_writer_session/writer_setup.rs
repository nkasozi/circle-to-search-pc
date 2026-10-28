use anyhow::{anyhow, bail, Result};
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_av_foundation::{
    AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVFileTypeMPEG4,
    AVMediaTypeVideo,
};
use objc2_core_media::CMTime;

use super::super::foundation_helpers::{describe_error, file_url, required_static};
use super::super::system_constant_names::{AV_FILE_TYPE_MPEG4_NAME, AV_MEDIA_TYPE_VIDEO_NAME};
use super::settings::{build_source_attributes, build_video_settings};
use crate::core::models::VideoEncodingRequest;

const VIDEO_INPUT_REJECTED_ERROR: &str = "video input rejected by the writer";

pub fn writer_error_text(writer: &AVAssetWriter) -> String {
    unsafe { writer.error() }
        .map(|error| describe_error(&error))
        .unwrap_or_default()
}

pub fn create_writer(request: &VideoEncodingRequest) -> Result<Retained<AVAssetWriter>> {
    let file_type = required_static(unsafe { AVFileTypeMPEG4 }, AV_FILE_TYPE_MPEG4_NAME)?;
    unsafe {
        AVAssetWriter::initWithURL_fileType_error(
            AVAssetWriter::alloc(),
            &file_url(&request.output_file_path),
            file_type,
        )
    }
    .map_err(|error| anyhow!("could not create video file: {}", describe_error(&error)))
}

pub fn create_video_input(request: &VideoEncodingRequest) -> Result<Retained<AVAssetWriterInput>> {
    let media_type = required_static(unsafe { AVMediaTypeVideo }, AV_MEDIA_TYPE_VIDEO_NAME)?;
    let video_settings = build_video_settings(request)?;
    let input = unsafe {
        AVAssetWriterInput::initWithMediaType_outputSettings(
            AVAssetWriterInput::alloc(),
            media_type,
            Some(&video_settings),
        )
    };
    unsafe { input.setExpectsMediaDataInRealTime(true) };
    Ok(input)
}

pub fn create_pixel_buffer_adaptor(
    input: &AVAssetWriterInput,
    request: &VideoEncodingRequest,
) -> Retained<AVAssetWriterInputPixelBufferAdaptor> {
    unsafe {
        AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(input, Some(&build_source_attributes(request)))
    }
}

pub fn attach_input_and_start_writing(
    writer: &AVAssetWriter,
    input: &AVAssetWriterInput,
    request: &VideoEncodingRequest,
) -> Result<bool> {
    if !unsafe { writer.canAddInput(input) } {
        bail!(VIDEO_INPUT_REJECTED_ERROR);
    }
    unsafe { writer.addInput(input) };
    unsafe { writer.setShouldOptimizeForNetworkUse(true) };
    if !unsafe { writer.startWriting() } {
        bail!("could not start writing: {}", writer_error_text(writer));
    }
    unsafe { writer.startSessionAtSourceTime(CMTime::new(0, request.frames_per_second as i32)) };
    Ok(true)
}
