use super::*;
use std::path::PathBuf;

fn build_request(pixel_layout: PixelLayout) -> VideoEncodingRequest {
    VideoEncodingRequest {
        output_file_path: PathBuf::from("/tmp/out.mp4"),
        dimensions: OutputDimensions {
            width: 1920,
            height: 1080,
        },
        pixel_layout,
        frames_per_second: 30,
    }
}

fn position_of(arguments: &[String], value: &str) -> Option<usize> {
    arguments.iter().position(|argument| argument == value)
}

#[test]
fn test_encode_arguments_describe_raw_input_and_output() {
    let arguments =
        build_encode_arguments(&build_request(PixelLayout::Nv12), VideoCodecChoice::Libx264);
    let pixel_format_index = position_of(&arguments, "-pix_fmt").unwrap_or(0);
    assert_eq!(arguments[pixel_format_index + 1], "nv12");
    assert!(arguments.contains(&"1920x1080".to_string()));
    assert!(arguments.contains(&"pipe:0".to_string()));
    assert!(arguments.contains(&"libx264".to_string()));
    assert_eq!(arguments.last().map(String::as_str), Some("/tmp/out.mp4"));
}

#[test]
fn test_encode_arguments_use_videotoolbox_with_bitrate() {
    let arguments = build_encode_arguments(
        &build_request(PixelLayout::Rgba),
        VideoCodecChoice::AppleVideoToolbox,
    );
    assert!(arguments.contains(&"h264_videotoolbox".to_string()));
    assert!(arguments.contains(&"4976k".to_string()));
    assert!(arguments.contains(&"rgba".to_string()));
}

#[test]
fn test_bitrate_is_clamped() {
    let tiny = OutputDimensions {
        width: 100,
        height: 100,
    };
    let huge = OutputDimensions {
        width: 7680,
        height: 4320,
    };
    assert_eq!(
        compute_video_bitrate_kilobits(tiny, 30),
        MINIMUM_BITRATE_KILOBITS
    );
    assert_eq!(
        compute_video_bitrate_kilobits(huge, 60),
        MAXIMUM_BITRATE_KILOBITS
    );
}

#[test]
fn test_mux_without_audio_maps_video_only() {
    let request = AudioMuxRequest {
        video_file_path: PathBuf::from("/tmp/v.mp4"),
        audio_file_paths: vec![],
        output_file_path: PathBuf::from("/tmp/final.mp4"),
    };
    let arguments = build_mux_arguments(&request);
    assert!(arguments.contains(&"0:v".to_string()));
    assert!(!arguments.contains(&"-filter_complex".to_string()));
}

#[test]
fn test_mux_with_single_audio_maps_it_directly() {
    let request = AudioMuxRequest {
        video_file_path: PathBuf::from("/tmp/v.mp4"),
        audio_file_paths: vec![PathBuf::from("/tmp/mic.wav")],
        output_file_path: PathBuf::from("/tmp/final.mp4"),
    };
    let arguments = build_mux_arguments(&request);
    assert!(arguments.contains(&"1:a".to_string()));
    assert!(arguments.contains(&"aac".to_string()));
}

#[test]
fn test_mux_with_two_audio_tracks_mixes_them() {
    let request = AudioMuxRequest {
        video_file_path: PathBuf::from("/tmp/v.mp4"),
        audio_file_paths: vec![
            PathBuf::from("/tmp/mic.wav"),
            PathBuf::from("/tmp/system.wav"),
        ],
        output_file_path: PathBuf::from("/tmp/final.mp4"),
    };
    let arguments = build_mux_arguments(&request);
    let filter_index = position_of(&arguments, "-filter_complex").unwrap_or(0);
    assert_eq!(
        arguments[filter_index + 1],
        "[1:a][2:a]amix=inputs=2:duration=longest:normalize=0[mixed_audio]"
    );
}

#[test]
fn test_thumbnail_arguments_extract_single_frame() {
    let arguments = build_thumbnail_arguments(Path::new("/tmp/v.mp4"), Path::new("/tmp/t.png"));
    assert!(arguments.contains(&"-frames:v".to_string()));
    assert_eq!(arguments.last().map(String::as_str), Some("/tmp/t.png"));
}
