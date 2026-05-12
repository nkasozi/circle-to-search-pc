use iced::widget::image;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebcamPipPosition {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl Default for WebcamPipPosition {
    fn default() -> Self {
        Self::BottomLeft
    }
}

#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub capture_region_x: i32,
    pub capture_region_y: i32,
    pub frames_per_second: u32,
    pub audio_enabled: bool,
    pub webcam_enabled: bool,
    pub webcam_pip_position: WebcamPipPosition,
    pub webcam_pip_size_percentage: f32,
    pub output_file_path: PathBuf,
}

impl RecordingConfig {
    pub fn build(
        capture_region_x: i32,
        capture_region_y: i32,
        frames_per_second: u32,
        audio_enabled: bool,
        webcam_enabled: bool,
        webcam_pip_position: WebcamPipPosition,
        webcam_pip_size_percentage: f32,
        output_file_path: PathBuf,
    ) -> Self {
        log::debug!(
            "building recording config: region=({}, {}), fps={}, audio={}, webcam={}",
            capture_region_x,
            capture_region_y,
            frames_per_second,
            audio_enabled,
            webcam_enabled,
        );

        Self {
            capture_region_x,
            capture_region_y,
            frames_per_second,
            audio_enabled,
            webcam_enabled,
            webcam_pip_position,
            webcam_pip_size_percentage,
            output_file_path,
        }
    }
}

#[derive(Clone)]
pub struct RecordingFrame {
    pub timestamp: Duration,
    pub width_pixels: u32,
    pub height_pixels: u32,
    pub raw_rgba_data: Vec<u8>,
}

impl std::fmt::Debug for RecordingFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecordingFrame")
            .field("timestamp", &self.timestamp)
            .field("width_pixels", &self.width_pixels)
            .field("height_pixels", &self.height_pixels)
            .field("data_length", &self.raw_rgba_data.len())
            .finish()
    }
}

impl RecordingFrame {
    pub fn build(
        timestamp: Duration,
        width_pixels: u32,
        height_pixels: u32,
        raw_rgba_data: Vec<u8>,
    ) -> Self {
        Self {
            timestamp,
            width_pixels,
            height_pixels,
            raw_rgba_data,
        }
    }

    pub fn to_image_handle(&self) -> image::Handle {
        image::Handle::from_rgba(
            self.width_pixels,
            self.height_pixels,
            self.raw_rgba_data.clone(),
        )
    }

    pub fn expected_data_length(&self) -> usize {
        (self.width_pixels as usize) * (self.height_pixels as usize) * 4
    }
}

#[derive(Clone)]
pub struct AudioSample {
    pub timestamp: Duration,
    pub pcm_data: Vec<f32>,
    pub sample_rate: u32,
    pub channel_count: u16,
}

impl std::fmt::Debug for AudioSample {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AudioSample")
            .field("timestamp", &self.timestamp)
            .field("sample_count", &self.pcm_data.len())
            .field("sample_rate", &self.sample_rate)
            .field("channel_count", &self.channel_count)
            .finish()
    }
}

impl AudioSample {
    pub fn build(
        timestamp: Duration,
        pcm_data: Vec<f32>,
        sample_rate: u32,
        channel_count: u16,
    ) -> Self {
        Self {
            timestamp,
            pcm_data,
            sample_rate,
            channel_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl AnnotationColor {
    pub fn from_rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn red() -> Self {
        Self::from_rgba(1.0, 0.0, 0.0, 1.0)
    }

    pub fn yellow() -> Self {
        Self::from_rgba(1.0, 1.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::from_rgba(1.0, 1.0, 1.0, 1.0)
    }

    pub fn to_rgba_bytes(&self) -> [u8; 4] {
        [
            (self.red * 255.0) as u8,
            (self.green * 255.0) as u8,
            (self.blue * 255.0) as u8,
            (self.alpha * 255.0) as u8,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationPoint {
    pub x: f32,
    pub y: f32,
}

impl AnnotationPoint {
    pub fn at(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawAnnotation {
    PenStroke {
        points: Vec<AnnotationPoint>,
        color: AnnotationColor,
        thickness: f32,
    },
    Arrow {
        start: AnnotationPoint,
        end: AnnotationPoint,
        color: AnnotationColor,
        thickness: f32,
    },
    Rectangle {
        top_left: AnnotationPoint,
        bottom_right: AnnotationPoint,
        color: AnnotationColor,
        thickness: f32,
    },
    TextLabel {
        position: AnnotationPoint,
        content: String,
        color: AnnotationColor,
        font_size: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationTool {
    Pen,
    Arrow,
    Rectangle,
    Text,
}

impl Default for AnnotationTool {
    fn default() -> Self {
        Self::Pen
    }
}

#[derive(Debug, Clone)]
pub struct EncodingConfig {
    pub width_pixels: u32,
    pub height_pixels: u32,
    pub frames_per_second: u32,
    pub audio_sample_rate: u32,
    pub audio_channel_count: u16,
    pub output_file_path: PathBuf,
}

impl EncodingConfig {
    pub fn build(
        width_pixels: u32,
        height_pixels: u32,
        frames_per_second: u32,
        audio_sample_rate: u32,
        audio_channel_count: u16,
        output_file_path: PathBuf,
    ) -> Self {
        Self {
            width_pixels,
            height_pixels,
            frames_per_second,
            audio_sample_rate,
            audio_channel_count,
            output_file_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_equality() {
        assert_eq!(RecordingState::Idle, RecordingState::Idle);
        assert_ne!(RecordingState::Idle, RecordingState::Recording);
        assert_ne!(RecordingState::Recording, RecordingState::Paused);
        assert_ne!(RecordingState::Paused, RecordingState::Stopped);
    }

    #[test]
    fn test_webcam_pip_position_default_is_bottom_left() {
        let default_position = WebcamPipPosition::default();
        assert_eq!(default_position, WebcamPipPosition::BottomLeft);
    }

    #[test]
    fn test_recording_config_build_stores_all_fields() {
        let config = RecordingConfig::build(
            100,
            200,
            30,
            true,
            false,
            WebcamPipPosition::TopRight,
            0.2,
            PathBuf::from("/tmp/output.mp4"),
        );

        assert_eq!(config.capture_region_x, 100);
        assert_eq!(config.capture_region_y, 200);
        assert_eq!(config.frames_per_second, 30);
        assert!(config.audio_enabled);
        assert!(!config.webcam_enabled);
        assert_eq!(config.webcam_pip_position, WebcamPipPosition::TopRight);
        assert_eq!(config.webcam_pip_size_percentage, 0.2);
        assert_eq!(config.output_file_path, PathBuf::from("/tmp/output.mp4"));
    }

    #[test]
    fn test_recording_frame_build_stores_all_fields() {
        let raw_data = vec![255u8; 100 * 50 * 4];
        let frame = RecordingFrame::build(Duration::from_millis(500), 100, 50, raw_data.clone());

        assert_eq!(frame.timestamp, Duration::from_millis(500));
        assert_eq!(frame.width_pixels, 100);
        assert_eq!(frame.height_pixels, 50);
        assert_eq!(frame.raw_rgba_data.len(), raw_data.len());
    }

    #[test]
    fn test_recording_frame_expected_data_length() {
        let frame =
            RecordingFrame::build(Duration::from_secs(0), 640, 480, vec![0u8; 640 * 480 * 4]);

        assert_eq!(frame.expected_data_length(), 640 * 480 * 4);
    }

    #[test]
    fn test_audio_sample_build_stores_all_fields() {
        let pcm_data = vec![0.5f32; 1024];
        let sample = AudioSample::build(Duration::from_millis(100), pcm_data.clone(), 44100, 2);

        assert_eq!(sample.timestamp, Duration::from_millis(100));
        assert_eq!(sample.pcm_data.len(), 1024);
        assert_eq!(sample.sample_rate, 44100);
        assert_eq!(sample.channel_count, 2);
    }

    #[test]
    fn test_annotation_color_from_rgba_stores_components() {
        let color = AnnotationColor::from_rgba(0.5, 0.6, 0.7, 0.8);

        assert_eq!(color.red, 0.5);
        assert_eq!(color.green, 0.6);
        assert_eq!(color.blue, 0.7);
        assert_eq!(color.alpha, 0.8);
    }

    #[test]
    fn test_annotation_color_to_rgba_bytes_converts_correctly() {
        let color = AnnotationColor::from_rgba(1.0, 0.5, 0.0, 1.0);
        let bytes = color.to_rgba_bytes();

        assert_eq!(bytes[0], 255);
        assert_eq!(bytes[1], 127);
        assert_eq!(bytes[2], 0);
        assert_eq!(bytes[3], 255);
    }

    #[test]
    fn test_annotation_color_presets() {
        let red = AnnotationColor::red();
        assert_eq!(red.red, 1.0);
        assert_eq!(red.green, 0.0);
        assert_eq!(red.blue, 0.0);

        let yellow = AnnotationColor::yellow();
        assert_eq!(yellow.red, 1.0);
        assert_eq!(yellow.green, 1.0);
        assert_eq!(yellow.blue, 0.0);

        let white = AnnotationColor::white();
        assert_eq!(white.red, 1.0);
        assert_eq!(white.green, 1.0);
        assert_eq!(white.blue, 1.0);
    }

    #[test]
    fn test_annotation_point_at_creates_point() {
        let point = AnnotationPoint::at(10.5, 20.3);

        assert_eq!(point.x, 10.5);
        assert_eq!(point.y, 20.3);
    }

    #[test]
    fn test_draw_annotation_pen_stroke_stores_fields() {
        let annotation = DrawAnnotation::PenStroke {
            points: vec![
                AnnotationPoint::at(0.0, 0.0),
                AnnotationPoint::at(10.0, 10.0),
            ],
            color: AnnotationColor::red(),
            thickness: 3.0,
        };

        match annotation {
            DrawAnnotation::PenStroke {
                points,
                color,
                thickness,
            } => {
                assert_eq!(points.len(), 2);
                assert_eq!(color, AnnotationColor::red());
                assert_eq!(thickness, 3.0);
            }
            _ => panic!("expected PenStroke variant"),
        }
    }

    #[test]
    fn test_draw_annotation_arrow_stores_fields() {
        let annotation = DrawAnnotation::Arrow {
            start: AnnotationPoint::at(0.0, 0.0),
            end: AnnotationPoint::at(100.0, 50.0),
            color: AnnotationColor::yellow(),
            thickness: 2.0,
        };

        match annotation {
            DrawAnnotation::Arrow {
                start,
                end,
                color,
                thickness,
            } => {
                assert_eq!(start, AnnotationPoint::at(0.0, 0.0));
                assert_eq!(end, AnnotationPoint::at(100.0, 50.0));
                assert_eq!(color, AnnotationColor::yellow());
                assert_eq!(thickness, 2.0);
            }
            _ => panic!("expected Arrow variant"),
        }
    }

    #[test]
    fn test_draw_annotation_rectangle_stores_fields() {
        let annotation = DrawAnnotation::Rectangle {
            top_left: AnnotationPoint::at(10.0, 20.0),
            bottom_right: AnnotationPoint::at(110.0, 120.0),
            color: AnnotationColor::white(),
            thickness: 1.5,
        };

        match annotation {
            DrawAnnotation::Rectangle {
                top_left,
                bottom_right,
                color,
                thickness,
            } => {
                assert_eq!(top_left, AnnotationPoint::at(10.0, 20.0));
                assert_eq!(bottom_right, AnnotationPoint::at(110.0, 120.0));
                assert_eq!(color, AnnotationColor::white());
                assert_eq!(thickness, 1.5);
            }
            _ => panic!("expected Rectangle variant"),
        }
    }

    #[test]
    fn test_draw_annotation_text_label_stores_fields() {
        let annotation = DrawAnnotation::TextLabel {
            position: AnnotationPoint::at(50.0, 75.0),
            content: "Hello".to_string(),
            color: AnnotationColor::red(),
            font_size: 16.0,
        };

        match annotation {
            DrawAnnotation::TextLabel {
                position,
                content,
                color,
                font_size,
            } => {
                assert_eq!(position, AnnotationPoint::at(50.0, 75.0));
                assert_eq!(content, "Hello");
                assert_eq!(color, AnnotationColor::red());
                assert_eq!(font_size, 16.0);
            }
            _ => panic!("expected TextLabel variant"),
        }
    }

    #[test]
    fn test_annotation_tool_default_is_pen() {
        let default_tool = AnnotationTool::default();
        assert_eq!(default_tool, AnnotationTool::Pen);
    }

    #[test]
    fn test_encoding_config_build_stores_all_fields() {
        let config =
            EncodingConfig::build(1920, 1080, 30, 44100, 2, PathBuf::from("/tmp/output.mp4"));

        assert_eq!(config.width_pixels, 1920);
        assert_eq!(config.height_pixels, 1080);
        assert_eq!(config.frames_per_second, 30);
        assert_eq!(config.audio_sample_rate, 44100);
        assert_eq!(config.audio_channel_count, 2);
        assert_eq!(config.output_file_path, PathBuf::from("/tmp/output.mp4"));
    }

    #[test]
    fn test_webcam_pip_position_serialization_roundtrip() {
        let positions = vec![
            WebcamPipPosition::BottomLeft,
            WebcamPipPosition::BottomRight,
            WebcamPipPosition::TopLeft,
            WebcamPipPosition::TopRight,
        ];

        for position in positions {
            let serialized = serde_json::to_string(&position).unwrap();
            let deserialized: WebcamPipPosition = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, position);
        }
    }
}
