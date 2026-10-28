use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_av_foundation::{
    AVAsset, AVAssetReader, AVAssetReaderOutput, AVAssetTrack, AVAssetWriterInput, AVMediaType,
    AVURLAsset,
};
use objc2_core_media::CMFormatDescription;

use super::foundation_helpers::{describe_error, file_url};

const SAMPLE_APPEND_WAIT_LIMIT: Duration = Duration::from_secs(10);
const SAMPLE_APPEND_POLL_INTERVAL: Duration = Duration::from_millis(1);
const MISSING_TRACK_ERROR: &str = "media file has no track of the requested type";
const SAMPLE_REJECTED_ERROR: &str = "a media sample was rejected while combining audio and video";
const SAMPLE_APPEND_TIMEOUT_ERROR: &str = "timed out while combining audio and video";
#[cfg(test)]
const VIDEO_MEDIA_TYPE_MISSING_MESSAGE: &str = "video media type";
#[cfg(test)]
const AUDIO_MEDIA_TYPE_MISSING_MESSAGE: &str = "audio media type";
#[cfg(test)]
const MISSING_TRACK_SECONDS: f64 = -1.0;

pub struct ThreadBound<T>(pub T);
unsafe impl<T> Send for ThreadBound<T> {}

impl<T> ThreadBound<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub fn open_asset(path: &Path) -> Retained<AVURLAsset> {
    unsafe { AVURLAsset::URLAssetWithURL_options(&file_url(path), None) }
}

#[allow(deprecated)]
pub fn first_track(asset: &AVAsset, media_type: &AVMediaType) -> Result<Retained<AVAssetTrack>> {
    unsafe { asset.tracksWithMediaType(media_type) }
        .firstObject()
        .context(MISSING_TRACK_ERROR)
}

pub fn first_format_description(track: &AVAssetTrack) -> Option<&CMFormatDescription> {
    let descriptions = unsafe { track.formatDescriptions() };
    let first_description = descriptions.firstObject()?;
    let description_pointer = Retained::into_raw(first_description) as *const CMFormatDescription;
    unsafe { description_pointer.as_ref() }
}

pub fn create_reader(asset: &AVAsset) -> Result<Retained<AVAssetReader>> {
    unsafe { AVAssetReader::initWithAsset_error(AVAssetReader::alloc(), asset) }
        .map_err(|error| anyhow!("could not read media: {}", describe_error(&error)))
}

pub fn pump_samples(output: &AVAssetReaderOutput, input: &AVAssetWriterInput) -> Result<u64> {
    let mut appended_count = 0u64;
    while let Some(sample_buffer) = unsafe { output.copyNextSampleBuffer() } {
        wait_until_input_ready(input)?;
        if !unsafe { input.appendSampleBuffer(&sample_buffer) } {
            bail!(SAMPLE_REJECTED_ERROR);
        }
        appended_count += 1;
    }
    unsafe { input.markAsFinished() };
    Ok(appended_count)
}

fn wait_until_input_ready(input: &AVAssetWriterInput) -> Result<()> {
    let started_at = Instant::now();
    while !unsafe { input.isReadyForMoreMediaData() } {
        if started_at.elapsed() > SAMPLE_APPEND_WAIT_LIMIT {
            bail!(SAMPLE_APPEND_TIMEOUT_ERROR);
        }
        std::thread::sleep(SAMPLE_APPEND_POLL_INTERVAL);
    }
    Ok(())
}

#[cfg(test)]
pub fn probe_track_seconds(path: &Path, media_type: &AVMediaType) -> f64 {
    let asset = open_asset(path);
    first_track(&asset, media_type)
        .map(|track| unsafe { track.timeRange() }.duration)
        .map(|duration| duration.value as f64 / duration.timescale.max(1) as f64)
        .unwrap_or(MISSING_TRACK_SECONDS)
}

#[cfg(test)]
pub fn probe_video_and_audio_seconds(path: &Path) -> (f64, f64) {
    let video_type =
        unsafe { objc2_av_foundation::AVMediaTypeVideo }.expect(VIDEO_MEDIA_TYPE_MISSING_MESSAGE);
    let audio_type =
        unsafe { objc2_av_foundation::AVMediaTypeAudio }.expect(AUDIO_MEDIA_TYPE_MISSING_MESSAGE);
    (
        probe_track_seconds(path, video_type),
        probe_track_seconds(path, audio_type),
    )
}
