#[allow(dead_code)]
mod audio_capturer;
mod mouse_position_provider;
mod screen_capturer;
#[allow(dead_code)]
mod screen_frame_capturer;
#[allow(dead_code)]
mod webcam_capturer;

#[allow(dead_code)]
pub use audio_capturer::AudioCapturer;
pub use mouse_position_provider::MousePositionProvider;
pub use screen_capturer::ScreenCapturer;
#[allow(dead_code)]
pub use screen_frame_capturer::ScreenFrameCapturer;
#[allow(dead_code)]
pub use webcam_capturer::WebcamCapturer;
