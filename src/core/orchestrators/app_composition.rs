use std::sync::Arc;

use crate::adapters::{CpalMicrophoneSource, DesktopFileLauncher, SystemPointerStateProvider};
use crate::core::models::{GlobalHotkeyAction, HotkeyBinding, UserSettings};
use crate::core::orchestrators::recording_flow::RecordingDependencies;
use crate::ports::{GlobalHotkeyListener, HotkeyApplyOutcome};

const LOG_TAG: &str = "[APP]";
const EVENT_GLOBAL_HOTKEY_LISTENER_CREATION_FAILED: &str = "global_hotkey_listener_creation_failed";

pub fn build_hotkey_bindings(settings: &UserSettings) -> Vec<HotkeyBinding> {
    vec![
        HotkeyBinding {
            action: GlobalHotkeyAction::Capture,
            accelerator: settings.capture_hotkey.clone(),
        },
        HotkeyBinding {
            action: GlobalHotkeyAction::ToggleRecording,
            accelerator: settings.recording_hotkey.clone(),
        },
    ]
}

pub fn build_hotkey_listener() -> Option<GlobalHotkeyListener> {
    GlobalHotkeyListener::build()
        .map_err(|creation_error| {
            log::error!(
                "{} {}", LOG_TAG,
                serde_json::json!({"event": EVENT_GLOBAL_HOTKEY_LISTENER_CREATION_FAILED, "error": creation_error.to_string()})
            );
            creation_error
        })
        .ok()
}

pub fn apply_hotkey_bindings(
    listener: Option<&mut GlobalHotkeyListener>,
    bindings: &[HotkeyBinding],
) -> HotkeyApplyOutcome {
    match listener {
        Some(active_listener) => active_listener.apply_bindings(bindings),
        None => HotkeyApplyOutcome::default(),
    }
}

#[cfg(target_os = "macos")]
pub fn build_recording_dependencies() -> RecordingDependencies {
    use crate::adapters::macos_media::{AvFoundationCameraSource, ScreenCaptureKitSource};
    use crate::adapters::NativeVideoEncoder;
    RecordingDependencies {
        screen_source: Arc::new(ScreenCaptureKitSource),
        microphone_source: Arc::new(CpalMicrophoneSource),
        camera_source: Arc::new(AvFoundationCameraSource),
        encoder: Arc::new(NativeVideoEncoder),
        pointer_provider: Arc::new(SystemPointerStateProvider),
        file_launcher: Arc::new(DesktopFileLauncher),
        permission_gate: Arc::new(crate::adapters::SystemCapturePermissionGate),
    }
}

#[cfg(target_os = "windows")]
fn build_platform_encoder() -> Arc<dyn crate::core::interfaces::ports::VideoEncoder> {
    Arc::new(crate::adapters::MediaFoundationVideoEncoder)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn build_platform_encoder() -> Arc<dyn crate::core::interfaces::ports::VideoEncoder> {
    Arc::new(crate::adapters::FfmpegVideoEncoder)
}

#[cfg(not(target_os = "macos"))]
pub fn build_recording_dependencies() -> RecordingDependencies {
    use crate::adapters::{NokhwaCameraSource, XcapScreenFrameSource};
    RecordingDependencies {
        screen_source: Arc::new(XcapScreenFrameSource),
        microphone_source: Arc::new(CpalMicrophoneSource),
        camera_source: Arc::new(NokhwaCameraSource),
        encoder: build_platform_encoder(),
        pointer_provider: Arc::new(SystemPointerStateProvider),
        file_launcher: Arc::new(DesktopFileLauncher),
        permission_gate: Arc::new(crate::adapters::SystemCapturePermissionGate),
    }
}

#[cfg(target_os = "macos")]
pub fn build_media_player() -> Arc<dyn crate::core::interfaces::ports::MediaPlayer> {
    Arc::new(crate::adapters::macos_media::AvFoundationMediaPlayer)
}

#[cfg(target_os = "windows")]
pub fn build_media_player() -> Arc<dyn crate::core::interfaces::ports::MediaPlayer> {
    Arc::new(
        crate::core::playback::decoded_media_player::DecodedMediaPlayer {
            decoder_factory: Arc::new(
                crate::adapters::windows_media::MediaFoundationDecoderFactory,
            ),
            audio_output: Arc::new(crate::adapters::CpalAudioOutput),
        },
    )
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn build_media_player() -> Arc<dyn crate::core::interfaces::ports::MediaPlayer> {
    Arc::new(
        crate::core::playback::decoded_media_player::DecodedMediaPlayer {
            decoder_factory: Arc::new(crate::adapters::FfmpegDecoderFactory),
            audio_output: Arc::new(crate::adapters::CpalAudioOutput),
        },
    )
}
