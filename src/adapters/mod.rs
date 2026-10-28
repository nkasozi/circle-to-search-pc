pub mod auto_launch;
mod camera_frame_processing;
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod cpal_audio_output;
mod cpal_microphone_source;
mod desktop_file_launcher;
mod ffmpeg;
mod file_system_recording_library;
mod google_lens_search_provider;
mod imgbb_image_hosting_service;
pub mod macos_app_behavior;
#[cfg(target_os = "macos")]
pub mod macos_media;
pub mod macos_permissions;
#[cfg(not(target_os = "macos"))]
mod nokhwa_camera_source;
mod system_capture_permission_gate;
mod system_pointer_state_provider;
mod tesseract_ocr_service;
#[cfg(target_os = "windows")]
pub mod windows_media;
mod xcap_screen_frame_source;

#[cfg(not(target_os = "macos"))]
pub use cpal_audio_output::CpalAudioOutput;
pub use cpal_microphone_source::CpalMicrophoneSource;
pub use desktop_file_launcher::DesktopFileLauncher;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use ffmpeg::{FfmpegDecoderFactory, FfmpegVideoEncoder};
pub use file_system_recording_library::FileSystemRecordingLibrary;
pub use google_lens_search_provider::GoogleLensSearchProvider;
pub use imgbb_image_hosting_service::ImgbbImageHostingService;
#[cfg(target_os = "macos")]
pub use macos_media::NativeVideoEncoder;
#[cfg(not(target_os = "macos"))]
pub use nokhwa_camera_source::NokhwaCameraSource;
pub use system_capture_permission_gate::SystemCapturePermissionGate;
pub use system_pointer_state_provider::SystemPointerStateProvider;
pub use tesseract_ocr_service::TesseractOcrService;
#[cfg(target_os = "windows")]
pub use windows_media::MediaFoundationVideoEncoder;
#[cfg(not(target_os = "macos"))]
pub use xcap_screen_frame_source::XcapScreenFrameSource;
