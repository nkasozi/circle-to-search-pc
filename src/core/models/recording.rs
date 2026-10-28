use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelLayout {
    Nv12,
    Rgba,
}

impl PixelLayout {
    pub fn expected_byte_length(self, width: u32, height: u32) -> usize {
        let pixel_count = width as usize * height as usize;
        match self {
            PixelLayout::Nv12 => pixel_count + pixel_count / 2,
            PixelLayout::Rgba => pixel_count * 4,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pixel_layout: PixelLayout,
    pub data: Arc<Vec<u8>>,
}

impl std::fmt::Debug for VideoFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VideoFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixel_layout", &self.pixel_layout)
            .field("byte_length", &self.data.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureArea {
    FullDisplay,
    Region(LogicalRect),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenCaptureRequest {
    pub display_origin_x: i32,
    pub display_origin_y: i32,
    pub capture_area: CaptureArea,
    pub output: OutputDimensions,
    pub frames_per_second: u32,
    pub shows_cursor: bool,
    pub captures_system_audio: bool,
    pub visible_own_window_titles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioChunk {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub interleaved_samples: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioTrackKind {
    Microphone,
    SystemAudio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoEncodingRequest {
    pub output_file_path: PathBuf,
    pub dimensions: OutputDimensions,
    pub pixel_layout: PixelLayout,
    pub frames_per_second: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioMuxRequest {
    pub video_file_path: PathBuf,
    pub audio_file_paths: Vec<PathBuf>,
    pub output_file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedRecording {
    pub file_path: PathBuf,
    pub duration: Duration,
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nv12_byte_length_is_one_and_a_half_bytes_per_pixel() {
        assert_eq!(PixelLayout::Nv12.expected_byte_length(4, 2), 12);
    }

    #[test]
    fn test_rgba_byte_length_is_four_bytes_per_pixel() {
        assert_eq!(PixelLayout::Rgba.expected_byte_length(4, 2), 32);
        assert_eq!(PixelLayout::Rgba.expected_byte_length(0, 2), 0);
    }
}
