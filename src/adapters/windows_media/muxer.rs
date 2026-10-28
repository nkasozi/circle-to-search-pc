use std::path::PathBuf;

use anyhow::{Context, Result};
use windows::Win32::Media::MediaFoundation::{
    IMFSample, IMFSinkWriter, IMFSourceReader, MFCreateSinkWriterFromURL,
    MFCreateSourceReaderFromURL, MF_SOURCE_READERF_ENDOFSTREAM,
    MF_SOURCE_READER_FIRST_VIDEO_STREAM,
};

use super::audio_track::AudioTrackWriter;
use super::runtime::{as_pcwstr, ensure_media_foundation, wide_path};
use crate::core::models::AudioMuxRequest;
use crate::core::recording::wav_reader::{mix_tracks, read_float_wav, DecodedAudio};

const LOG_TAG: &str = "[MEDIA_FOUNDATION]";
const RECORDING_MUXED_EVENT: &str = "recording_muxed";
const SAVE_RECORDING_FAILED_ERROR: &str = "failed to save recording";
const READ_RECORDED_VIDEO_FAILED_ERROR: &str = "could not read the recorded video";
const CREATE_RECORDING_FILE_FAILED_ERROR: &str = "could not create the recording file";
const FINISH_RECORDING_FAILED_ERROR: &str = "could not finish the recording";

pub fn mux_recording(request: &AudioMuxRequest) -> Result<PathBuf> {
    ensure_media_foundation()?;
    let _ = std::fs::remove_file(&request.output_file_path);
    if request.audio_file_paths.is_empty() {
        std::fs::copy(&request.video_file_path, &request.output_file_path)
            .context(SAVE_RECORDING_FAILED_ERROR)?;
        return Ok(request.output_file_path.clone());
    }
    let tracks = request
        .audio_file_paths
        .iter()
        .map(|path| read_float_wav(path))
        .collect::<Result<Vec<DecodedAudio>>>()?;
    let audio = mix_tracks(tracks)?;
    combine(request, audio)?;
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": RECORDING_MUXED_EVENT})
    );
    Ok(request.output_file_path.clone())
}

fn combine(request: &AudioMuxRequest, audio: DecodedAudio) -> Result<()> {
    let video_path = wide_path(&request.video_file_path);
    let output_path = wide_path(&request.output_file_path);
    let reader = unsafe { MFCreateSourceReaderFromURL(as_pcwstr(&video_path), None) }
        .context(READ_RECORDED_VIDEO_FAILED_ERROR)?;
    let writer = unsafe { MFCreateSinkWriterFromURL(as_pcwstr(&output_path), None, None) }
        .context(CREATE_RECORDING_FILE_FAILED_ERROR)?;
    let video_stream_selector = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
    let native_video_type = unsafe { reader.GetNativeMediaType(video_stream_selector, 0) }?;
    let video_stream = unsafe { writer.AddStream(&native_video_type) }?;
    unsafe { writer.SetInputMediaType(video_stream, &native_video_type, None) }?;
    let mut audio_track = AudioTrackWriter::attach(&writer, audio)?;
    unsafe { writer.BeginWriting() }?;
    let last_timestamp = copy_video_samples(&reader, &writer, video_stream, &mut audio_track)?;
    audio_track.write_until(&writer, last_timestamp)?;
    unsafe { writer.Finalize() }.context(FINISH_RECORDING_FAILED_ERROR)?;
    Ok(())
}

fn copy_video_samples(
    reader: &IMFSourceReader,
    writer: &IMFSinkWriter,
    video_stream: u32,
    audio_track: &mut AudioTrackWriter,
) -> Result<i64> {
    let video_stream_selector = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
    let mut last_timestamp = 0i64;
    loop {
        let mut stream_flags = 0u32;
        let mut timestamp = 0i64;
        let mut sample: Option<IMFSample> = None;
        unsafe {
            reader.ReadSample(
                video_stream_selector,
                0,
                None,
                Some(&mut stream_flags),
                Some(&mut timestamp),
                Some(&mut sample),
            )
        }?;
        if stream_flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
            return Ok(last_timestamp);
        }
        let Some(video_sample) = sample else {
            continue;
        };
        let duration = unsafe { video_sample.GetSampleDuration() }.unwrap_or(0);
        last_timestamp = timestamp + duration;
        audio_track.write_until(writer, timestamp)?;
        unsafe { writer.WriteSample(video_stream, &video_sample) }?;
    }
}
