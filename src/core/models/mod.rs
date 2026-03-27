mod capture_buffer;
mod ocr;
mod screen_region;
pub mod user_settings;
mod window_info;

pub use capture_buffer::CaptureBuffer;
pub use ocr::{DetectedText, DetectedWord, OcrResult};
pub use screen_region::ScreenRegion;
pub use user_settings::{ImageHostingAuthMode, ImageUploadHttpMethod, ThemeMode, UserSettings};
pub use window_info::WindowInfo;
