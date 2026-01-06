mod capture_buffer;
mod ocr;
mod screen_region;
pub mod user_settings;

pub use capture_buffer::CaptureBuffer;
pub use ocr::{DetectedText, DetectedWord, OcrResult};
pub use screen_region::ScreenRegion;
pub use user_settings::{ThemeMode, UserSettings};
