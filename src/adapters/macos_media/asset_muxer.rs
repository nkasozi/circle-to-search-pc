mod composition;
mod readers;
mod settings;
mod writer;

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use objc2_av_foundation::{AVMediaTypeAudio, AVMediaTypeVideo};

use super::asset_reading::{first_track, open_asset, pump_samples, ThreadBound};
use super::foundation_helpers::required_static;
use super::system_constant_names::{AV_MEDIA_TYPE_AUDIO_NAME, AV_MEDIA_TYPE_VIDEO_NAME};
use crate::core::models::AudioMuxRequest;
use composition::build_audio_composition;
use readers::{create_mux_readers, MuxReaders};
use writer::{create_mux_writer, finish_writer, start_all, MuxWriter};

const LOG_TAG: &str = "[ASSET_MUXER]";
const RECORDING_MUXED_EVENT: &str = "recording_muxed";
const MUX_SAMPLES_APPENDED_EVENT: &str = "mux_samples_appended";
const SAVE_RECORDING_FAILED_ERROR: &str = "failed to save recording";
const AUDIO_THREAD_PANICKED_ERROR: &str = "audio thread panicked";

pub fn mux_recording(request: &AudioMuxRequest) -> Result<PathBuf> {
    let _ = std::fs::remove_file(&request.output_file_path);
    if request.audio_file_paths.is_empty() {
        std::fs::copy(&request.video_file_path, &request.output_file_path)
            .context(SAVE_RECORDING_FAILED_ERROR)?;
        return Ok(request.output_file_path.clone());
    }
    combine_video_with_audio(request)?;
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": RECORDING_MUXED_EVENT, "audio_tracks": request.audio_file_paths.len()})
    );
    Ok(request.output_file_path.clone())
}

#[allow(deprecated)]
fn combine_video_with_audio(request: &AudioMuxRequest) -> Result<()> {
    let video_type = required_static(unsafe { AVMediaTypeVideo }, AV_MEDIA_TYPE_VIDEO_NAME)?;
    let audio_type = required_static(unsafe { AVMediaTypeAudio }, AV_MEDIA_TYPE_AUDIO_NAME)?;
    let video_asset = open_asset(&request.video_file_path);
    let video_track = first_track(&video_asset, video_type)?;
    let video_duration = unsafe { video_asset.duration() };
    let composition = build_audio_composition(&request.audio_file_paths, video_duration)?;
    let readers = create_mux_readers(&video_asset, &video_track, &composition, audio_type)?;
    let mux_writer = create_mux_writer(request, video_type, audio_type, &video_track)?;
    start_all(
        &mux_writer.writer,
        &readers.video_reader,
        &readers.audio_reader,
    )?;
    let (video_samples, audio_samples) = pump_all_samples(&readers, &mux_writer)?;
    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": MUX_SAMPLES_APPENDED_EVENT, "video": video_samples, "audio": audio_samples})
    );
    finish_writer(&mux_writer.writer)
}

fn pump_all_samples(readers: &MuxReaders, mux_writer: &MuxWriter) -> Result<(u64, u64)> {
    let audio_pump_handle = {
        let bound = ThreadBound((readers.audio_output.clone(), mux_writer.audio_input.clone()));
        std::thread::spawn(move || {
            let (output, input) = bound.into_inner();
            pump_samples(&output, &input)
        })
    };
    let video_samples = pump_samples(&readers.video_output, &mux_writer.video_input)?;
    let audio_samples = audio_pump_handle
        .join()
        .map_err(|_| anyhow!(AUDIO_THREAD_PANICKED_ERROR))??;
    Ok((video_samples, audio_samples))
}
