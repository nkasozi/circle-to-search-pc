mod asset_muxer;
mod asset_reading;
mod asset_writer_session;
mod av_player;
mod camera_permission;
mod camera_source;
mod foundation_helpers;
mod native_video_encoder;
mod pixel_buffers;
mod screen_capture_kit;
mod system_constant_names;
mod thumbnail;

pub use av_player::AvFoundationMediaPlayer;
pub use camera_permission::{ensure_capture_access, CaptureDeviceKind};
pub use camera_source::AvFoundationCameraSource;
pub use native_video_encoder::NativeVideoEncoder;
pub use screen_capture_kit::ScreenCaptureKitSource;

#[cfg(test)]
pub use asset_reading::probe_video_and_audio_seconds;
