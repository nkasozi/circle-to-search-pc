use super::*;
use crate::core::models::ImageUploadHttpMethod;

#[test]
fn test_validate_image_hosting_settings_rejects_invalid_provider_url() {
    let mut settings = UserSettings::default();
    settings.image_hosting_provider_url = "not-a-valid-url".to_string();
    let result = AppOrchestrator::validate_image_hosting_settings(&settings);
    assert!(result.is_err());
}

#[test]
fn test_validate_image_hosting_settings_rejects_non_numeric_expiration() {
    let mut settings = UserSettings::default();
    settings.image_hosting_expiration_seconds = "abc".to_string();
    let result = AppOrchestrator::validate_image_hosting_settings(&settings);
    assert!(result.is_err());
}

#[test]
fn test_should_rebuild_search_provider_when_image_hosting_config_changes() {
    let previous_settings = UserSettings::default();
    let mut next_settings = previous_settings.clone();
    next_settings.image_hosting_public_key_value = "updated-key".to_string();

    let should_rebuild =
        AppOrchestrator::should_rebuild_search_provider(&previous_settings, &next_settings);

    assert!(should_rebuild);
}

#[test]
fn test_is_using_default_public_key_returns_true_for_default_settings() {
    let settings = UserSettings::default();
    assert!(settings.is_using_default_public_key());
}

#[test]
fn test_is_using_default_public_key_returns_false_for_custom_key() {
    let mut settings = UserSettings::default();
    settings.image_hosting_public_key_value = "my-custom-api-key".to_string();
    assert!(!settings.is_using_default_public_key());
}

#[test]
fn test_should_rebuild_search_provider_when_http_method_changes() {
    let previous_settings = UserSettings::default();
    let mut next_settings = previous_settings.clone();
    next_settings.image_hosting_http_method = ImageUploadHttpMethod::Get;

    let should_rebuild =
        AppOrchestrator::should_rebuild_search_provider(&previous_settings, &next_settings);

    assert!(should_rebuild);
}

#[test]
fn test_should_rebuild_search_provider_when_image_field_name_changes() {
    let previous_settings = UserSettings::default();
    let mut next_settings = previous_settings.clone();
    next_settings.image_hosting_image_field_name = "photo".to_string();

    let should_rebuild =
        AppOrchestrator::should_rebuild_search_provider(&previous_settings, &next_settings);

    assert!(should_rebuild);
}

#[test]
fn test_user_settings_default_http_method_is_post() {
    let settings = UserSettings::default();
    assert!(matches!(
        settings.image_hosting_http_method,
        ImageUploadHttpMethod::Post
    ));
}

#[test]
fn test_user_settings_default_image_field_name_is_image() {
    let settings = UserSettings::default();
    assert_eq!(settings.image_hosting_image_field_name, "image");
}
