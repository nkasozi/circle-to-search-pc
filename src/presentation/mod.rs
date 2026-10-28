mod capture_view;
pub mod design;
pub mod home_view;
mod interactive_ocr_view;
mod onboarding_view;
pub mod player_view;
pub mod recording;
pub mod settings_view;
mod window_picker_view;

pub use capture_view::{CaptureView, CaptureViewMessage};
pub use interactive_ocr_view::{DrawStroke, InteractiveOcrMessage, InteractiveOcrView};
pub use onboarding_view::{OnboardingMessage, OnboardingSetup, OnboardingView};
pub use window_picker_view::{WindowPickerMessage, WindowPickerView};
