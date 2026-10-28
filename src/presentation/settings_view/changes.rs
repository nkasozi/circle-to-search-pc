use std::path::PathBuf;

use crate::core::models::{
    CameraBubbleShape, CameraBubbleSize, CountdownChoice, FrameRateChoice, GlobalHotkeyAction,
    ImageHostingAuthMode, ImageUploadHttpMethod, RecordingQuality, ThemeMode, UserSettings,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsChange {
    Shortcut(GlobalHotkeyAction, String),
    Theme(ThemeMode),
    LaunchAtLogin(bool),
    ShowHomeOnLaunch(bool),
    FrameRate(FrameRateChoice),
    Quality(RecordingQuality),
    Countdown(CountdownChoice),
    ClickHighlights(bool),
    CursorSpotlight(bool),
    CameraBubbleSize(CameraBubbleSize),
    CameraBubbleShape(CameraBubbleShape),
    MicrophoneByDefault(bool),
    SystemAudioByDefault(bool),
    RecordingsFolder(PathBuf),
    SearchUrl(String),
    HostingProviderUrl(String),
    HostingAuthMode(ImageHostingAuthMode),
    HostingKeyName(String),
    HostingKeyValue(String),
    HostingExpirationSeconds(String),
    HostingHttpMethod(ImageUploadHttpMethod),
    HostingImageFieldName(String),
}

pub fn apply_settings_change(settings: &mut UserSettings, change: SettingsChange) -> bool {
    let recording = &mut settings.recording_preferences;
    match change {
        SettingsChange::Shortcut(GlobalHotkeyAction::Capture, accelerator) => {
            settings.capture_hotkey = accelerator
        }
        SettingsChange::Shortcut(GlobalHotkeyAction::ToggleRecording, accelerator) => {
            settings.recording_hotkey = accelerator
        }
        SettingsChange::Theme(theme_mode) => settings.theme_mode = theme_mode,
        SettingsChange::LaunchAtLogin(enabled) => settings.launch_at_login = enabled,
        SettingsChange::ShowHomeOnLaunch(enabled) => settings.show_home_on_launch = enabled,
        SettingsChange::FrameRate(frame_rate) => recording.frame_rate = frame_rate,
        SettingsChange::Quality(quality) => recording.quality = quality,
        SettingsChange::Countdown(countdown) => recording.countdown = countdown,
        SettingsChange::ClickHighlights(enabled) => recording.click_highlights_enabled = enabled,
        SettingsChange::CursorSpotlight(enabled) => recording.cursor_spotlight_enabled = enabled,
        SettingsChange::CameraBubbleSize(size) => recording.camera_bubble_size = size,
        SettingsChange::CameraBubbleShape(shape) => recording.camera_bubble_shape = shape,
        SettingsChange::MicrophoneByDefault(enabled) => recording.microphone_enabled = enabled,
        SettingsChange::SystemAudioByDefault(enabled) => recording.system_audio_enabled = enabled,
        SettingsChange::RecordingsFolder(folder) => {
            settings.recording_save_location = folder.to_string_lossy().to_string()
        }
        SettingsChange::SearchUrl(url) => settings.image_search_url_template = url,
        SettingsChange::HostingProviderUrl(url) => settings.image_hosting_provider_url = url,
        SettingsChange::HostingAuthMode(mode) => settings.image_hosting_auth_mode = mode,
        SettingsChange::HostingKeyName(name) => settings.image_hosting_public_key_name = name,
        SettingsChange::HostingKeyValue(value) if !value.trim().is_empty() => {
            settings.image_hosting_public_key_value = value
        }
        SettingsChange::HostingKeyValue(_) => return false,
        SettingsChange::HostingExpirationSeconds(seconds) => {
            settings.image_hosting_expiration_seconds = seconds
        }
        SettingsChange::HostingHttpMethod(method) => settings.image_hosting_http_method = method,
        SettingsChange::HostingImageFieldName(name) => {
            settings.image_hosting_image_field_name = name
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_changes_target_the_right_action() {
        let mut settings = UserSettings::default();
        apply_settings_change(
            &mut settings,
            SettingsChange::Shortcut(GlobalHotkeyAction::ToggleRecording, "Ctrl+KeyR".into()),
        );
        assert_eq!(settings.recording_hotkey, "Ctrl+KeyR");
        assert_eq!(
            settings.capture_hotkey,
            UserSettings::default().capture_hotkey
        );
    }

    #[test]
    fn test_recording_preferences_are_updated() {
        let mut settings = UserSettings::default();
        apply_settings_change(
            &mut settings,
            SettingsChange::FrameRate(FrameRateChoice::Sixty),
        );
        apply_settings_change(
            &mut settings,
            SettingsChange::CameraBubbleShape(CameraBubbleShape::RoundedSquare),
        );
        assert_eq!(
            settings.recording_preferences.frame_rate,
            FrameRateChoice::Sixty
        );
        assert_eq!(
            settings.recording_preferences.camera_bubble_shape,
            CameraBubbleShape::RoundedSquare
        );
    }

    #[test]
    fn test_blank_public_key_is_ignored() {
        let mut settings = UserSettings::default();
        let original_key = settings.image_hosting_public_key_value.clone();
        assert!(!apply_settings_change(
            &mut settings,
            SettingsChange::HostingKeyValue("   ".into())
        ));
        assert_eq!(settings.image_hosting_public_key_value, original_key);
    }
}
