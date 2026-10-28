use anyhow::{anyhow, bail, Result};
use objc2::rc::Retained;
use objc2::{AllocAnyThread, Message};
use objc2_av_foundation::{
    AVAssetReader, AVAssetTrack, AVAssetWriter, AVAssetWriterInput, AVAssetWriterStatus,
    AVFileTypeMPEG4, AVMediaType,
};
use objc2_core_media::CMTime;

use super::super::asset_reading::first_format_description;
use super::super::foundation_helpers::{
    describe_error, file_url, required_static, wait_for_unit_completion,
};
use super::super::system_constant_names::AV_FILE_TYPE_MPEG4_NAME;
use super::settings::{audio_settings, AAC_FORMAT_IDENTIFIER};
use crate::core::models::AudioMuxRequest;

const START_WRITING_FAILED_ERROR: &str = "could not start writing the recording";
const READ_RECORDED_MEDIA_FAILED_ERROR: &str = "could not read the recorded media";

pub struct MuxWriter {
    pub writer: Retained<AVAssetWriter>,
    pub video_input: Retained<AVAssetWriterInput>,
    pub audio_input: Retained<AVAssetWriterInput>,
}

fn create_output_writer(request: &AudioMuxRequest) -> Result<Retained<AVAssetWriter>> {
    let file_type = required_static(unsafe { AVFileTypeMPEG4 }, AV_FILE_TYPE_MPEG4_NAME)?;
    unsafe {
        AVAssetWriter::initWithURL_fileType_error(
            AVAssetWriter::alloc(),
            &file_url(&request.output_file_path),
            file_type,
        )
    }
    .map_err(|error| {
        anyhow!(
            "could not create recording file: {}",
            describe_error(&error)
        )
    })
}

pub fn create_mux_writer(
    request: &AudioMuxRequest,
    video_type: &AVMediaType,
    audio_type: &AVMediaType,
    video_track: &AVAssetTrack,
) -> Result<MuxWriter> {
    let writer = create_output_writer(request)?;
    let video_input = unsafe {
        AVAssetWriterInput::initWithMediaType_outputSettings_sourceFormatHint(
            AVAssetWriterInput::alloc(),
            video_type,
            None,
            first_format_description(video_track),
        )
    };
    let aac_settings = audio_settings(AAC_FORMAT_IDENTIFIER)?;
    let audio_input = unsafe {
        AVAssetWriterInput::initWithMediaType_outputSettings(
            AVAssetWriterInput::alloc(),
            audio_type,
            Some(&aac_settings),
        )
    };
    unsafe {
        writer.addInput(&video_input);
        writer.addInput(&audio_input);
        writer.setShouldOptimizeForNetworkUse(true);
    }
    Ok(MuxWriter {
        writer,
        video_input,
        audio_input,
    })
}

pub fn start_all(
    writer: &AVAssetWriter,
    video_reader: &AVAssetReader,
    audio_reader: &AVAssetReader,
) -> Result<()> {
    if !unsafe { writer.startWriting() } {
        bail!(START_WRITING_FAILED_ERROR);
    }
    unsafe { writer.startSessionAtSourceTime(CMTime::new(0, 1)) };
    if !unsafe { video_reader.startReading() } || !unsafe { audio_reader.startReading() } {
        bail!(READ_RECORDED_MEDIA_FAILED_ERROR);
    }
    Ok(())
}

pub fn finish_writer(writer: &AVAssetWriter) -> Result<()> {
    let bound_writer = writer.retain();
    wait_for_unit_completion(|completion| unsafe {
        bound_writer.finishWritingWithCompletionHandler(completion)
    })?;
    if unsafe { writer.status() } != AVAssetWriterStatus::Completed {
        let message = unsafe { writer.error() }
            .map(|error| describe_error(&error))
            .unwrap_or_default();
        bail!("could not finish the recording: {}", message);
    }
    Ok(())
}
