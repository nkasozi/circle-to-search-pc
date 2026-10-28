use anyhow::Result;
use objc2::rc::Retained;
use objc2_av_foundation::{
    AVVideoAverageBitRateKey, AVVideoCodecKey, AVVideoCodecTypeH264,
    AVVideoCompressionPropertiesKey, AVVideoExpectedSourceFrameRateKey, AVVideoHeightKey,
    AVVideoMaxKeyFrameIntervalKey, AVVideoWidthKey,
};
use objc2_core_video::{
    kCVPixelBufferHeightKey, kCVPixelBufferPixelFormatTypeKey, kCVPixelBufferWidthKey,
};
use objc2_foundation::{NSDictionary, NSString};

use super::super::foundation_helpers::{
    bridged_key, build_dictionary, dictionary_object, number_object, required_static, string_object,
};
use super::super::pixel_buffers::core_video_pixel_format;
use super::super::system_constant_names::{
    AV_VIDEO_AVERAGE_BIT_RATE_KEY_NAME, AV_VIDEO_CODEC_KEY_NAME, AV_VIDEO_CODEC_TYPE_H264_NAME,
    AV_VIDEO_COMPRESSION_PROPERTIES_KEY_NAME, AV_VIDEO_EXPECTED_SOURCE_FRAME_RATE_KEY_NAME,
    AV_VIDEO_HEIGHT_KEY_NAME, AV_VIDEO_MAX_KEY_FRAME_INTERVAL_KEY_NAME, AV_VIDEO_WIDTH_KEY_NAME,
};
use crate::core::models::VideoEncodingRequest;

const BITS_PER_PIXEL_FRAME: f64 = 0.12;
const MINIMUM_BITRATE: f64 = 3_000_000.0;
const MAXIMUM_BITRATE: f64 = 60_000_000.0;

type SettingsDictionary = Retained<NSDictionary<NSString, objc2::runtime::AnyObject>>;

pub fn compute_bitrate(request: &VideoEncodingRequest) -> i64 {
    let pixel_rate = request.dimensions.width as f64
        * request.dimensions.height as f64
        * request.frames_per_second as f64;
    (pixel_rate * BITS_PER_PIXEL_FRAME).clamp(MINIMUM_BITRATE, MAXIMUM_BITRATE) as i64
}

fn build_compression_properties(request: &VideoEncodingRequest) -> Result<SettingsDictionary> {
    Ok(build_dictionary(&[
        (
            required_static(
                unsafe { AVVideoAverageBitRateKey },
                AV_VIDEO_AVERAGE_BIT_RATE_KEY_NAME,
            )?,
            number_object(compute_bitrate(request)),
        ),
        (
            required_static(
                unsafe { AVVideoExpectedSourceFrameRateKey },
                AV_VIDEO_EXPECTED_SOURCE_FRAME_RATE_KEY_NAME,
            )?,
            number_object(request.frames_per_second as i64),
        ),
        (
            required_static(
                unsafe { AVVideoMaxKeyFrameIntervalKey },
                AV_VIDEO_MAX_KEY_FRAME_INTERVAL_KEY_NAME,
            )?,
            number_object(request.frames_per_second as i64 * 2),
        ),
    ]))
}

pub fn build_video_settings(request: &VideoEncodingRequest) -> Result<SettingsDictionary> {
    let compression = build_compression_properties(request)?;
    Ok(build_dictionary(&[
        (
            required_static(unsafe { AVVideoCodecKey }, AV_VIDEO_CODEC_KEY_NAME)?,
            string_object(required_static(
                unsafe { AVVideoCodecTypeH264 },
                AV_VIDEO_CODEC_TYPE_H264_NAME,
            )?),
        ),
        (
            required_static(unsafe { AVVideoWidthKey }, AV_VIDEO_WIDTH_KEY_NAME)?,
            number_object(request.dimensions.width as i64),
        ),
        (
            required_static(unsafe { AVVideoHeightKey }, AV_VIDEO_HEIGHT_KEY_NAME)?,
            number_object(request.dimensions.height as i64),
        ),
        (
            required_static(
                unsafe { AVVideoCompressionPropertiesKey },
                AV_VIDEO_COMPRESSION_PROPERTIES_KEY_NAME,
            )?,
            dictionary_object(compression),
        ),
    ]))
}

pub fn build_source_attributes(request: &VideoEncodingRequest) -> SettingsDictionary {
    build_dictionary(&[
        (
            bridged_key(unsafe { kCVPixelBufferPixelFormatTypeKey }),
            number_object(core_video_pixel_format(request.pixel_layout) as i64),
        ),
        (
            bridged_key(unsafe { kCVPixelBufferWidthKey }),
            number_object(request.dimensions.width as i64),
        ),
        (
            bridged_key(unsafe { kCVPixelBufferHeightKey }),
            number_object(request.dimensions.height as i64),
        ),
    ])
}
