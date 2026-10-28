use anyhow::Result;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_avf_audio::{AVEncoderBitRateKey, AVFormatIDKey, AVNumberOfChannelsKey, AVSampleRateKey};
use objc2_foundation::{NSDictionary, NSString};

use super::super::foundation_helpers::{build_dictionary, number_object, required_static};
use super::super::system_constant_names::{
    AV_ENCODER_BIT_RATE_KEY_NAME, AV_FORMAT_ID_KEY_NAME, AV_NUMBER_OF_CHANNELS_KEY_NAME,
    AV_SAMPLE_RATE_KEY_NAME,
};

pub const AAC_FORMAT_IDENTIFIER: i64 = 0x6161_6320;
pub const LINEAR_PCM_FORMAT_IDENTIFIER: i64 = 0x6C70_636D;
const OUTPUT_SAMPLE_RATE: i64 = 48_000;
const OUTPUT_CHANNEL_COUNT: i64 = 2;
const OUTPUT_AUDIO_BITRATE: i64 = 192_000;

pub fn audio_settings(
    format_identifier: i64,
) -> Result<Retained<NSDictionary<NSString, AnyObject>>> {
    let mut entries = vec![
        (
            required_static(unsafe { AVFormatIDKey }, AV_FORMAT_ID_KEY_NAME)?,
            number_object(format_identifier),
        ),
        (
            required_static(unsafe { AVSampleRateKey }, AV_SAMPLE_RATE_KEY_NAME)?,
            number_object(OUTPUT_SAMPLE_RATE),
        ),
        (
            required_static(
                unsafe { AVNumberOfChannelsKey },
                AV_NUMBER_OF_CHANNELS_KEY_NAME,
            )?,
            number_object(OUTPUT_CHANNEL_COUNT),
        ),
    ];
    if format_identifier == AAC_FORMAT_IDENTIFIER {
        entries.push((
            required_static(unsafe { AVEncoderBitRateKey }, AV_ENCODER_BIT_RATE_KEY_NAME)?,
            number_object(OUTPUT_AUDIO_BITRATE),
        ));
    }
    Ok(build_dictionary(&entries))
}
