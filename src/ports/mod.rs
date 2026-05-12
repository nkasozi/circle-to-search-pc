#[allow(dead_code)]
mod cpal_audio_capturer;
mod keyboard_listener;
mod mouse_position_provider;
#[allow(dead_code)]
mod nokhwa_webcam_capturer;
pub mod system_tray;
mod xcap_screen_capturer;
#[allow(dead_code)]
mod xcap_screen_frame_capturer;

#[allow(unused_imports)]
pub use cpal_audio_capturer::CpalAudioCapturer;
pub use keyboard_listener::{GlobalKeyboardEvent, GlobalKeyboardListener};
pub use mouse_position_provider::SystemMousePositionProvider;
#[allow(unused_imports)]
pub use nokhwa_webcam_capturer::NokhwaWebcamCapturer;
pub use system_tray::{SystemTray, TrayEvent};
pub use xcap_screen_capturer::XcapScreenCapturer;
#[allow(unused_imports)]
pub use xcap_screen_frame_capturer::XcapScreenFrameCapturer;
