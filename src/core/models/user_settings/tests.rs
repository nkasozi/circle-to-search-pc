use super::*;

fn build_example_settings() -> UserSettings {
    UserSettings {
        image_search_url_template: "https://example.com/{IMAGE_URL}".to_string(),
        image_hosting_provider_url: "https://api.example.com/upload".to_string(),
        image_hosting_auth_mode: ImageHostingAuthMode::Header,
        image_hosting_public_key_name: "X-API-Key".to_string(),
        image_hosting_public_key_value: "example-key".to_string(),
        image_hosting_expiration_seconds: "300".to_string(),
        image_hosting_http_method: ImageUploadHttpMethod::Post,
        image_hosting_image_field_name: "image".to_string(),
        capture_hotkey: "ctrl+shift+a".to_string(),
        theme_mode: ThemeMode::Light,
        show_home_on_launch: true,
        onboarding_complete: true,
        launch_at_login: true,
        install_id: Some("test-id".to_string()),
        screenshot_save_location: "/tmp/screenshots".to_string(),
        recording_preferences: RecordingPreferences::default(),
        recording_save_location: UserSettings::default_recording_save_location(),
        recording_hotkey: UserSettings::default_recording_hotkey(),
    }
}

fn build_roundtrip_settings() -> UserSettings {
    UserSettings {
        image_search_url_template: "https://test.com/{IMAGE_URL}".to_string(),
        image_hosting_provider_url: "https://api.test.com/upload".to_string(),
        image_hosting_auth_mode: ImageHostingAuthMode::Header,
        image_hosting_public_key_name: "X-Test-Key".to_string(),
        image_hosting_public_key_value: "test-key".to_string(),
        image_hosting_expiration_seconds: "120".to_string(),
        image_hosting_http_method: ImageUploadHttpMethod::Post,
        image_hosting_image_field_name: "image".to_string(),
        capture_hotkey: "ctrl+shift+t".to_string(),
        theme_mode: ThemeMode::Light,
        show_home_on_launch: true,
        onboarding_complete: true,
        launch_at_login: true,
        install_id: Some("test-roundtrip-id".to_string()),
        screenshot_save_location: "/tmp/test-screenshots".to_string(),
        recording_preferences: RecordingPreferences::default(),
        recording_save_location: UserSettings::default_recording_save_location(),
        recording_hotkey: UserSettings::default_recording_hotkey(),
    }
}

fn assert_image_hosting_fields_match(actual: &UserSettings, expected: &UserSettings) -> bool {
    assert_eq!(
        actual.image_search_url_template,
        expected.image_search_url_template
    );
    assert_eq!(
        actual.image_hosting_provider_url,
        expected.image_hosting_provider_url
    );
    assert_eq!(
        actual.image_hosting_auth_mode,
        expected.image_hosting_auth_mode
    );
    assert_eq!(
        actual.image_hosting_public_key_name,
        expected.image_hosting_public_key_name
    );
    assert_eq!(
        actual.image_hosting_public_key_value,
        expected.image_hosting_public_key_value
    );
    assert_eq!(
        actual.image_hosting_expiration_seconds,
        expected.image_hosting_expiration_seconds
    );
    true
}

fn assert_preference_fields_match(actual: &UserSettings, expected: &UserSettings) -> bool {
    assert_eq!(actual.capture_hotkey, expected.capture_hotkey);
    assert_eq!(actual.theme_mode, expected.theme_mode);
    assert_eq!(actual.show_home_on_launch, expected.show_home_on_launch);
    assert_eq!(actual.onboarding_complete, expected.onboarding_complete);
    assert_eq!(actual.launch_at_login, expected.launch_at_login);
    true
}

fn assert_default_image_hosting_values(settings: &UserSettings) -> bool {
    assert_eq!(
        settings.image_hosting_provider_url,
        global_constants::IMGBB_API_URL
    );
    assert_eq!(
        settings.image_hosting_public_key_name,
        global_constants::IMGBB_PUBLIC_KEY_QUERY_NAME
    );
    assert_eq!(
        settings.image_hosting_public_key_value,
        global_constants::IMGBB_PUBLIC_KEY
    );
    assert_eq!(
        settings.image_hosting_expiration_seconds,
        global_constants::IMGBB_EXPIRATION_SECONDS
    );
    assert_eq!(
        settings.image_hosting_auth_mode,
        ImageHostingAuthMode::Query
    );
    true
}

#[test]
fn test_theme_mode_default_is_dark() {
    let default_theme = ThemeMode::default();
    assert_eq!(default_theme, ThemeMode::Dark);
}

#[test]
fn test_theme_mode_serialization() {
    let theme = ThemeMode::Dark;
    let serialized = serde_json::to_string(&theme).unwrap();
    assert_eq!(serialized, "\"Dark\"");
}

#[test]
fn test_theme_mode_deserialization() {
    let json = "\"Light\"";
    let theme: ThemeMode = serde_json::from_str(json).unwrap();
    assert_eq!(theme, ThemeMode::Light);
}

#[test]
fn test_user_settings_default_values() {
    let settings = UserSettings::default();

    assert_eq!(
        settings.image_search_url_template,
        global_constants::DEFAULT_IMAGE_SEARCH_URL
    );
    assert!(assert_default_image_hosting_values(&settings));
    assert_eq!(
        settings.capture_hotkey,
        global_constants::DEFAULT_CAPTURE_HOTKEY
    );
    assert_eq!(settings.theme_mode, ThemeMode::Dark);
    assert!(settings.show_home_on_launch);
    assert!(!settings.onboarding_complete);
    assert!(!settings.launch_at_login);
}

#[test]
fn test_user_settings_serialization() {
    let settings = build_example_settings();

    let serialized = serde_json::to_string(&settings).unwrap();
    let deserialized: UserSettings = serde_json::from_str(&serialized).unwrap();

    assert!(assert_image_hosting_fields_match(&deserialized, &settings));
    assert!(assert_preference_fields_match(&deserialized, &settings));
}

#[test]
fn test_user_settings_deserialization_with_missing_show_home_on_launch_defaults_to_true() {
    let json = r#"{
        "image_search_url_template": "https://example.com",
        "capture_hotkey": "ctrl+a",
        "theme_mode": "Dark"
    }"#;

    let settings: UserSettings = serde_json::from_str(json).unwrap();
    assert!(settings.show_home_on_launch);
    assert!(assert_default_image_hosting_values(&settings));
}

#[test]
fn test_user_settings_save_and_load_roundtrip() {
    let temporary_directory = std::env::temp_dir().join("circle-to-search-test");
    std::fs::create_dir_all(&temporary_directory).unwrap();
    let original_settings = build_roundtrip_settings();

    let test_file = temporary_directory.join("test_settings.json");
    let contents = serde_json::to_string_pretty(&original_settings).unwrap();
    std::fs::write(&test_file, contents).unwrap();

    let loaded_contents = std::fs::read_to_string(&test_file).unwrap();
    let loaded_settings: UserSettings = serde_json::from_str(&loaded_contents).unwrap();

    assert!(assert_image_hosting_fields_match(
        &loaded_settings,
        &original_settings
    ));
    assert!(assert_preference_fields_match(
        &loaded_settings,
        &original_settings
    ));

    std::fs::remove_dir_all(&temporary_directory).ok();
}
