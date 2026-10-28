#[cfg(test)]
mod tests;

use std::path::Path;

use super::flags::{
    AAC_CODEC, AUDIO_BITRATE_FLAG, AUDIO_CODEC_FLAG, CONSTANT_RATE_FACTOR_FLAG,
    CONSTANT_RATE_FACTOR_VALUE, COPY_CODEC, ENABLED_VALUE, FAST_START_MOVIE_FLAG,
    FILTER_COMPLEX_FLAG, FIRST_INPUT_VIDEO_STREAM, FORMAT_FLAG, FRAME_RATE_FLAG, HIDE_BANNER_FLAG,
    INPUT_FLAG, KEYFRAME_INTERVAL_FLAG, LIBX264_CODEC, LOG_LEVEL_ERROR, LOG_LEVEL_FLAG, MAP_FLAG,
    MOVIE_FLAGS_FLAG, NV12_PIXEL_FORMAT, OVERWRITE_FLAG, PIXEL_FORMAT_FLAG, PRESET_FLAG,
    RAW_VIDEO_FORMAT, REALTIME_FLAG, RGBA_PIXEL_FORMAT, SECOND_INPUT_AUDIO_STREAM, SEEK_FLAG,
    SHORTEST_FLAG, SINGLE_FRAME_COUNT, SIZE_FLAG, STANDARD_INPUT_PIPE, VERY_FAST_PRESET,
    VIDEOTOOLBOX_H264_CODEC, VIDEO_BITRATE_FLAG, VIDEO_CODEC_FLAG, VIDEO_FILTER_FLAG,
    VIDEO_FRAME_COUNT_FLAG, YUV420P_PIXEL_FORMAT,
};
use crate::core::models::{AudioMuxRequest, OutputDimensions, PixelLayout, VideoEncodingRequest};

const BITS_PER_PIXEL_FRAME: f64 = 0.08;
const MINIMUM_BITRATE_KILOBITS: u64 = 2_000;
const MAXIMUM_BITRATE_KILOBITS: u64 = 40_000;
const THUMBNAIL_SEEK_SECONDS: &str = "0.3";
const THUMBNAIL_SCALE_FILTER: &str = "scale=480:-2";
const AUDIO_BITRATE: &str = "192k";
const MIXED_AUDIO_LABEL: &str = "[mixed_audio]";
const COMMON_ARGUMENTS: [&str; 4] = [
    HIDE_BANNER_FLAG,
    LOG_LEVEL_FLAG,
    LOG_LEVEL_ERROR,
    OVERWRITE_FLAG,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodecChoice {
    AppleVideoToolbox,
    Libx264,
}

pub fn platform_video_codec() -> VideoCodecChoice {
    match cfg!(target_os = "macos") {
        true => VideoCodecChoice::AppleVideoToolbox,
        false => VideoCodecChoice::Libx264,
    }
}

pub fn compute_video_bitrate_kilobits(dimensions: OutputDimensions, frames_per_second: u32) -> u64 {
    let pixel_rate = dimensions.width as f64 * dimensions.height as f64 * frames_per_second as f64;
    let kilobits = (pixel_rate * BITS_PER_PIXEL_FRAME / 1000.0) as u64;
    kilobits.clamp(MINIMUM_BITRATE_KILOBITS, MAXIMUM_BITRATE_KILOBITS)
}

fn ffmpeg_pixel_format_name(pixel_layout: PixelLayout) -> &'static str {
    match pixel_layout {
        PixelLayout::Nv12 => NV12_PIXEL_FORMAT,
        PixelLayout::Rgba => RGBA_PIXEL_FORMAT,
    }
}

fn to_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

pub fn build_encode_arguments(
    request: &VideoEncodingRequest,
    codec: VideoCodecChoice,
) -> Vec<String> {
    let size = format!("{}x{}", request.dimensions.width, request.dimensions.height);
    let frame_rate = request.frames_per_second.to_string();
    let mut arguments = to_strings(&COMMON_ARGUMENTS);
    arguments.extend(to_strings(&[FORMAT_FLAG, RAW_VIDEO_FORMAT]));
    arguments.extend(to_strings(&[
        PIXEL_FORMAT_FLAG,
        ffmpeg_pixel_format_name(request.pixel_layout),
    ]));
    arguments.extend(to_strings(&[
        SIZE_FLAG,
        &size,
        FRAME_RATE_FLAG,
        &frame_rate,
        INPUT_FLAG,
        STANDARD_INPUT_PIPE,
    ]));
    arguments.extend(build_codec_arguments(request, codec));
    arguments.extend(to_strings(&[
        PIXEL_FORMAT_FLAG,
        YUV420P_PIXEL_FORMAT,
        MOVIE_FLAGS_FLAG,
        FAST_START_MOVIE_FLAG,
    ]));
    arguments.push(request.output_file_path.to_string_lossy().to_string());
    arguments
}

fn build_codec_arguments(request: &VideoEncodingRequest, codec: VideoCodecChoice) -> Vec<String> {
    let bitrate = format!(
        "{}k",
        compute_video_bitrate_kilobits(request.dimensions, request.frames_per_second)
    );
    let keyframe_interval = (request.frames_per_second * 2).to_string();
    match codec {
        VideoCodecChoice::AppleVideoToolbox => to_strings(&[
            VIDEO_CODEC_FLAG,
            VIDEOTOOLBOX_H264_CODEC,
            VIDEO_BITRATE_FLAG,
            &bitrate,
            REALTIME_FLAG,
            ENABLED_VALUE,
            KEYFRAME_INTERVAL_FLAG,
            &keyframe_interval,
        ]),
        VideoCodecChoice::Libx264 => to_strings(&[
            VIDEO_CODEC_FLAG,
            LIBX264_CODEC,
            PRESET_FLAG,
            VERY_FAST_PRESET,
            CONSTANT_RATE_FACTOR_FLAG,
            CONSTANT_RATE_FACTOR_VALUE,
            KEYFRAME_INTERVAL_FLAG,
            &keyframe_interval,
        ]),
    }
}

pub fn build_mux_arguments(request: &AudioMuxRequest) -> Vec<String> {
    let mut arguments = to_strings(&COMMON_ARGUMENTS);
    arguments.extend(to_strings(&[
        INPUT_FLAG,
        &request.video_file_path.to_string_lossy(),
    ]));
    for audio_file_path in &request.audio_file_paths {
        arguments.extend(to_strings(&[
            INPUT_FLAG,
            &audio_file_path.to_string_lossy(),
        ]));
    }
    arguments.extend(build_audio_mapping_arguments(
        request.audio_file_paths.len(),
    ));
    arguments.extend(to_strings(&[
        VIDEO_CODEC_FLAG,
        COPY_CODEC,
        AUDIO_CODEC_FLAG,
        AAC_CODEC,
        AUDIO_BITRATE_FLAG,
        AUDIO_BITRATE,
    ]));
    arguments.extend(to_strings(&[MOVIE_FLAGS_FLAG, FAST_START_MOVIE_FLAG]));
    arguments.push(request.output_file_path.to_string_lossy().to_string());
    arguments
}

fn build_audio_mix_arguments(track_count: usize) -> Vec<String> {
    let inputs: String = (1..=track_count)
        .map(|index| format!("[{}:a]", index))
        .collect();
    let filter = format!(
        "{}amix=inputs={}:duration=longest:normalize=0{}",
        inputs, track_count, MIXED_AUDIO_LABEL
    );
    to_strings(&[
        FILTER_COMPLEX_FLAG,
        &filter,
        MAP_FLAG,
        FIRST_INPUT_VIDEO_STREAM,
        MAP_FLAG,
        MIXED_AUDIO_LABEL,
        SHORTEST_FLAG,
    ])
}

fn build_audio_mapping_arguments(audio_track_count: usize) -> Vec<String> {
    match audio_track_count {
        0 => to_strings(&[MAP_FLAG, FIRST_INPUT_VIDEO_STREAM]),
        1 => to_strings(&[
            MAP_FLAG,
            FIRST_INPUT_VIDEO_STREAM,
            MAP_FLAG,
            SECOND_INPUT_AUDIO_STREAM,
            SHORTEST_FLAG,
        ]),
        track_count => build_audio_mix_arguments(track_count),
    }
}

pub fn build_thumbnail_arguments(video_file_path: &Path, thumbnail_path: &Path) -> Vec<String> {
    let mut arguments = to_strings(&COMMON_ARGUMENTS);
    arguments.extend(to_strings(&[
        SEEK_FLAG,
        THUMBNAIL_SEEK_SECONDS,
        INPUT_FLAG,
        &video_file_path.to_string_lossy(),
        VIDEO_FRAME_COUNT_FLAG,
        SINGLE_FRAME_COUNT,
        VIDEO_FILTER_FLAG,
        THUMBNAIL_SCALE_FILTER,
        &thumbnail_path.to_string_lossy(),
    ]));
    arguments
}
