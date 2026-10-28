use std::path::Path;
use std::time::Duration;

use super::flags::{
    AUDIO_CHANNELS_FLAG, AUDIO_SAMPLE_RATE_FLAG, DISABLE_AUDIO_FLAG, DISABLE_VIDEO_FLAG,
    FLOAT_32_LITTLE_ENDIAN_FORMAT, FORMAT_FLAG, HIDE_BANNER_FLAG, INPUT_FLAG, LOG_LEVEL_ERROR,
    LOG_LEVEL_FLAG, PIXEL_FORMAT_FLAG, RAW_VIDEO_FORMAT, RGBA_PIXEL_FORMAT, SEEK_FLAG,
    STANDARD_OUTPUT_TARGET, VIDEO_FILTER_FLAG,
};

pub const PLAYBACK_SAMPLE_RATE: u32 = 48_000;
pub const PLAYBACK_CHANNEL_COUNT: u16 = 2;

fn seconds_text(position: Duration) -> String {
    format!("{:.3}", position.as_secs_f64())
}

pub fn build_probe_arguments(file_path: &Path) -> Vec<String> {
    vec![
        HIDE_BANNER_FLAG.to_string(),
        INPUT_FLAG.to_string(),
        file_path.to_string_lossy().to_string(),
    ]
}

pub fn build_video_decode_arguments(
    file_path: &Path,
    start: Duration,
    width: u32,
    height: u32,
) -> Vec<String> {
    [
        HIDE_BANNER_FLAG,
        LOG_LEVEL_FLAG,
        LOG_LEVEL_ERROR,
        SEEK_FLAG,
        &seconds_text(start),
        INPUT_FLAG,
        &file_path.to_string_lossy(),
        DISABLE_AUDIO_FLAG,
        VIDEO_FILTER_FLAG,
        &format!("scale={}:{}", width, height),
        FORMAT_FLAG,
        RAW_VIDEO_FORMAT,
        PIXEL_FORMAT_FLAG,
        RGBA_PIXEL_FORMAT,
        STANDARD_OUTPUT_TARGET,
    ]
    .iter()
    .map(|value| value.to_string())
    .collect()
}

pub fn build_audio_decode_arguments(file_path: &Path, start: Duration) -> Vec<String> {
    [
        HIDE_BANNER_FLAG,
        LOG_LEVEL_FLAG,
        LOG_LEVEL_ERROR,
        SEEK_FLAG,
        &seconds_text(start),
        INPUT_FLAG,
        &file_path.to_string_lossy(),
        DISABLE_VIDEO_FLAG,
        FORMAT_FLAG,
        FLOAT_32_LITTLE_ENDIAN_FORMAT,
        AUDIO_CHANNELS_FLAG,
        &PLAYBACK_CHANNEL_COUNT.to_string(),
        AUDIO_SAMPLE_RATE_FLAG,
        &PLAYBACK_SAMPLE_RATE.to_string(),
        STANDARD_OUTPUT_TARGET,
    ]
    .iter()
    .map(|value| value.to_string())
    .collect()
}

pub fn scaled_even_size(width: u32, height: u32, maximum_width: u32) -> (u32, u32) {
    let target_width = width.min(maximum_width.max(2));
    let target_height = ((height as u64 * target_width as u64) / width.max(1) as u64) as u32;
    (target_width.max(2) & !1, target_height.max(2) & !1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_arguments_seek_scale_and_output_rgba() {
        let arguments = build_video_decode_arguments(
            Path::new("/tmp/a.mp4"),
            Duration::from_millis(1_500),
            1280,
            800,
        );
        assert!(arguments
            .windows(2)
            .any(|pair| pair[0] == "-ss" && pair[1] == "1.500"));
        assert!(arguments.contains(&"scale=1280:800".to_string()));
        assert!(arguments.contains(&"rgba".to_string()));
    }

    #[test]
    fn test_audio_arguments_request_stereo_float_at_playback_rate() {
        let arguments = build_audio_decode_arguments(Path::new("/tmp/a.mp4"), Duration::ZERO);
        assert!(arguments.contains(&"f32le".to_string()));
        assert!(arguments.contains(&"48000".to_string()));
        assert!(arguments.contains(&"-vn".to_string()));
    }

    #[test]
    fn test_scaled_even_size() {
        assert_eq!(scaled_even_size(3024, 1964, 1280), (1280, 830));
        assert_eq!(scaled_even_size(641, 361, 1280), (640, 360));
    }
}
