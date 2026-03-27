use super::*;
use crate::core::models::OcrResult;

struct MockScreenCapturer;
impl ScreenCapturer for MockScreenCapturer {
    fn capture_screen_at_region(&self, _region: &ScreenRegion) -> anyhow::Result<CaptureBuffer> {
        let raw_data = vec![255u8; 100 * 100 * 4];
        Ok(CaptureBuffer::build_from_raw_data(1.0, 100, 100, raw_data))
    }

    fn list_capturable_windows(&self) -> anyhow::Result<Vec<WindowInfo>> {
        Ok(vec![])
    }

    fn capture_window_by_id(&self, _window_id: u32) -> anyhow::Result<CaptureBuffer> {
        let raw_data = vec![255u8; 100 * 100 * 4];
        Ok(CaptureBuffer::build_from_raw_data(1.0, 100, 100, raw_data))
    }
}

struct MockMouseProvider;
impl MousePositionProvider for MockMouseProvider {
    fn get_current_mouse_position(&self) -> Result<ScreenRegion, String> {
        Ok(ScreenRegion::at_coordinates(0, 0))
    }
}

struct MockOcrService;
#[async_trait::async_trait]
impl OcrService for MockOcrService {
    async fn extract_text_from_image(
        &self,
        _image: &image::DynamicImage,
    ) -> anyhow::Result<OcrResult> {
        Ok(OcrResult {
            text_blocks: vec![],
            full_text: "test".to_string(),
        })
    }
}

struct MockSearchProvider;
#[async_trait::async_trait]
impl ReverseImageSearchProvider for MockSearchProvider {
    async fn perform_search(
        &self,
        _buffer: &CaptureBuffer,
        _query: Option<&str>,
    ) -> anyhow::Result<String> {
        Ok("https://test.com/search".to_string())
    }
}

fn create_test_orchestrator() -> AppOrchestrator {
    AppOrchestrator::build(
        Arc::new(MockScreenCapturer),
        Arc::new(MockMouseProvider),
        Arc::new(MockOcrService),
        Arc::new(MockSearchProvider),
        UserSettings::default(),
    )
}

#[test]
fn test_build_creates_orchestrator_with_correct_initial_state() {
    let orchestrator = create_test_orchestrator();

    assert_eq!(orchestrator.windows.len(), 0);
    assert!(orchestrator.main_window_id.is_none());
    assert!(orchestrator.settings_window_id.is_none());
    assert!(matches!(
        orchestrator.settings_edit_state,
        SettingsEditState::Closed
    ));
    assert!(!orchestrator.status.is_empty());
}

#[test]
fn test_handle_capture_error_updates_status() {
    let mut orchestrator = create_test_orchestrator();
    let error_message = "Test error".to_string();

    let _ = orchestrator.handle_capture_error(error_message.clone());

    assert!(orchestrator.status.contains("Test error"));
    assert!(orchestrator.status.contains("Capture failed"));
}

#[test]
fn test_handle_ocr_service_ready_updates_service() {
    let mut orchestrator = create_test_orchestrator();
    let new_service = Arc::new(MockOcrService) as Arc<dyn OcrService>;

    let _ = orchestrator.handle_ocr_service_ready(new_service);

    assert!(orchestrator.status.contains("Ready"));
}

#[test]
fn test_handle_ocr_service_failed_updates_status() {
    let mut orchestrator = create_test_orchestrator();
    let error = "OCR initialization failed".to_string();

    let _ = orchestrator.handle_ocr_service_failed(error.clone());

    assert!(orchestrator.status.contains("OCR initialization failed"));
}

#[test]
fn test_update_settings_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let new_url = "https://new.search.com?q={}".to_string();
    let _ = orchestrator.update(OrchestratorMessage::UpdateSearchUrl(new_url.clone()));
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
    let _ = orchestrator.update(OrchestratorMessage::UpdateHotkey(new_hotkey.clone()));
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
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingProviderUrl(
        new_provider_url.clone(),
    ));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_provider_url == new_provider_url
    ));
}

#[test]
fn test_update_image_hosting_auth_mode_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingAuthMode(
        ImageHostingAuthMode::Header,
    ));
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
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingPublicKeyName(
        new_public_key_name.clone(),
    ));
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
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingPublicKeyValue(
        new_public_key_value.clone(),
    ));
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
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingExpirationSeconds(
        new_expiration.clone(),
    ));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_expiration_seconds == new_expiration
    ));
}

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
fn test_build_clipboard_image_data_returns_original_buffer_without_drawings() {
    let capture_buffer = CaptureBuffer::build_from_raw_data(1.0, 2, 2, vec![1u8; 16]);

    let rgba_data = AppOrchestrator::build_clipboard_image_data(&capture_buffer, &[]);

    assert_eq!(rgba_data, capture_buffer.raw_data);
}

#[test]
fn test_update_theme_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(OrchestratorMessage::UpdateTheme(ThemeMode::Light));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if matches!(settings.theme_mode, ThemeMode::Light)
    ));
}

#[test]
fn test_get_window_title_returns_correct_title() {
    let orchestrator = create_test_orchestrator();
    let id = Id::unique();
    let title = orchestrator.get_window_title(id);
    assert_eq!(title, "Circle to Search");
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
fn test_update_image_hosting_public_key_value_empty_string_does_not_change_stored_key() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingPublicKeyValue(
        "".to_string(),
    ));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_public_key_value == UserSettings::default_image_hosting_public_key_value()
    ));
}

#[test]
fn test_update_image_hosting_http_method_modifies_temp_settings() {
    let mut orchestrator = create_test_orchestrator();
    orchestrator.settings_edit_state = SettingsEditState::Editing(UserSettings::default());
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingHttpMethod(
        ImageUploadHttpMethod::Put,
    ));
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
    let _ = orchestrator.update(OrchestratorMessage::UpdateImageHostingImageFieldName(
        new_field_name.clone(),
    ));
    assert!(matches!(
        &orchestrator.settings_edit_state,
        SettingsEditState::Editing(settings) if settings.image_hosting_image_field_name == new_field_name
    ));
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
