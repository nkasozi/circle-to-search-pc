use super::*;
use crate::core::models::{ImageHostingAuthMode, ImageUploadHttpMethod, ThemeMode};
use crate::presentation::settings_view::SettingsChange;

#[test]
fn test_update_settings_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_url = "https://new.search.com?q={}".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::SearchUrl(new_url.clone())));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_search_url_template == new_url
    ));
}

#[test]
fn test_update_hotkey_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_hotkey = "Ctrl+Shift+C".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::Shortcut(
        GlobalHotkeyAction::Capture,
        new_hotkey.clone(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.capture_hotkey == new_hotkey
    ));
}

#[test]
fn test_update_image_hosting_provider_url_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_provider_url = "https://api.example.com/upload".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::HostingProviderUrl(
        new_provider_url.clone(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_provider_url == new_provider_url
    ));
}

#[test]
fn test_update_image_hosting_auth_mode_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(settings_change(SettingsChange::HostingAuthMode(
        ImageHostingAuthMode::Header,
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_auth_mode == ImageHostingAuthMode::Header
    ));
}

#[test]
fn test_update_image_hosting_public_key_name_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_public_key_name = "X-Custom-Key".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::HostingKeyName(
        new_public_key_name.clone(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_public_key_name == new_public_key_name
    ));
}

#[test]
fn test_update_image_hosting_public_key_value_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_public_key_value = "new-value".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::HostingKeyValue(
        new_public_key_value.clone(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_public_key_value == new_public_key_value
    ));
}

#[test]
fn test_update_image_hosting_expiration_seconds_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_expiration = "180".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::HostingExpirationSeconds(
        new_expiration.clone(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_expiration_seconds == new_expiration
    ));
}

#[test]
fn test_update_theme_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(settings_change(SettingsChange::Theme(ThemeMode::Light)));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if matches!(settings.theme_mode, ThemeMode::Light)
    ));
}

#[test]
fn test_update_image_hosting_public_key_value_empty_string_does_not_change_stored_key() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(settings_change(SettingsChange::HostingKeyValue(
        "".to_string(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_public_key_value == UserSettings::default_image_hosting_public_key_value()
    ));
}

#[test]
fn test_update_image_hosting_http_method_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(settings_change(SettingsChange::HostingHttpMethod(
        ImageUploadHttpMethod::Put,
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if matches!(settings.image_hosting_http_method, ImageUploadHttpMethod::Put)
    ));
}

#[test]
fn test_update_image_hosting_image_field_name_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_field_name = "photo".to_string();
    let _ = orchestrator.update(settings_change(SettingsChange::HostingImageFieldName(
        new_field_name.clone(),
    )));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_image_field_name == new_field_name
    ));
}

fn settings_change(
    change: crate::presentation::settings_view::SettingsChange,
) -> OrchestratorMessage {
    OrchestratorMessage::Settings(crate::presentation::settings_view::SettingsMessage::Change(
        change,
    ))
}
