mod capture_buffer;
mod hotkey;
mod ocr;
mod recording;
mod recording_preferences;
mod recording_summary;
mod screen_region;
pub mod user_settings;
mod window_info;

pub use capture_buffer::CaptureBuffer;
pub use hotkey::{
    current_platform_key_symbol_style, format_accelerator_for_display, GlobalHotkeyAction,
    HotkeyBinding, HotkeyRegistration, HotkeyRegistrationFailure, KeySymbolStyle,
};
pub use ocr::{DetectedText, DetectedWord, OcrResult};
pub use recording::{
    AudioChunk, AudioMuxRequest, AudioTrackKind, CaptureArea, FinishedRecording, LogicalRect,
    OutputDimensions, PixelLayout, ScreenCaptureRequest, VideoEncodingRequest, VideoFrame,
};
pub use recording_preferences::{
    CameraBubbleShape, CameraBubbleSize, CaptureMode, CountdownChoice, FrameRateChoice,
    RecordingPreferences, RecordingQuality,
};
pub use recording_summary::{
    format_elapsed_clock, format_file_size, format_relative_time, RecordingSummary,
};
pub use screen_region::ScreenRegion;
pub use user_settings::{ImageHostingAuthMode, ImageUploadHttpMethod, ThemeMode, UserSettings};
pub use window_info::WindowInfo;
