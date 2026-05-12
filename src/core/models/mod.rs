mod capture_buffer;
mod ocr;
#[allow(dead_code)]
pub mod recording;
mod screen_region;
pub mod user_settings;
mod window_info;

pub use capture_buffer::CaptureBuffer;
pub use ocr::{DetectedText, DetectedWord, OcrResult};
#[allow(unused_imports)]
pub use recording::{
    AnnotationColor, AnnotationPoint, AnnotationTool, AudioSample, DrawAnnotation, EncodingConfig,
    RecordingConfig, RecordingFrame, RecordingState, WebcamPipPosition,
};
pub use screen_region::ScreenRegion;
pub use user_settings::{ImageHostingAuthMode, ImageUploadHttpMethod, ThemeMode, UserSettings};
pub use window_info::WindowInfo;
