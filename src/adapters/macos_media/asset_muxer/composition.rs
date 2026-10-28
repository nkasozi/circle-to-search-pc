use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use objc2::rc::Retained;
use objc2_av_foundation::{AVMediaTypeAudio, AVMutableComposition};
use objc2_core_media::{kCMPersistentTrackID_Invalid, CMTime, CMTimeRange};

use super::super::asset_reading::{first_track, open_asset};
use super::super::foundation_helpers::{describe_error, required_static};
use super::super::system_constant_names::AV_MEDIA_TYPE_AUDIO_NAME;

const AUDIO_TRACK_NOT_ADDED_ERROR: &str = "could not add an audio track";

pub fn build_audio_composition(
    audio_file_paths: &[PathBuf],
    duration: CMTime,
) -> Result<Retained<AVMutableComposition>> {
    let composition = unsafe { AVMutableComposition::composition() };
    let audio_type = required_static(unsafe { AVMediaTypeAudio }, AV_MEDIA_TYPE_AUDIO_NAME)?;
    for audio_file_path in audio_file_paths {
        let audio_asset = open_asset(audio_file_path);
        let source_track = first_track(&audio_asset, audio_type)?;
        let composition_track = unsafe {
            composition.addMutableTrackWithMediaType_preferredTrackID(
                audio_type,
                kCMPersistentTrackID_Invalid,
            )
        }
        .context(AUDIO_TRACK_NOT_ADDED_ERROR)?;
        let available_duration = unsafe { audio_asset.duration() };
        let insert_range = CMTimeRange {
            start: unsafe { CMTime::new(0, 1) },
            duration: shorter_time(available_duration, duration),
        };
        unsafe {
            composition_track.insertTimeRange_ofTrack_atTime_error(
                insert_range,
                &source_track,
                CMTime::new(0, 1),
            )
        }
        .map_err(|error| anyhow!("could not add audio: {}", describe_error(&error)))?;
    }
    Ok(composition)
}

fn shorter_time(first: CMTime, second: CMTime) -> CMTime {
    let first_seconds = first.value as f64 / first.timescale.max(1) as f64;
    let second_seconds = second.value as f64 / second.timescale.max(1) as f64;
    match first_seconds < second_seconds {
        true => first,
        false => second,
    }
}
