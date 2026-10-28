use super::*;

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
fn test_build_clipboard_image_data_returns_original_buffer_without_drawings() {
    let capture_buffer = CaptureBuffer::build_from_raw_data(1.0, 2, 2, vec![1u8; 16]);

    let rgba_data = AppOrchestrator::build_clipboard_image_data(&capture_buffer, &[]);

    assert_eq!(rgba_data, capture_buffer.raw_data);
}

#[test]
fn test_get_window_title_returns_correct_title() {
    let orchestrator = create_test_orchestrator();
    let id = Id::unique();
    let title = orchestrator.get_window_title(id);
    assert_eq!(title, "Circle to Search");
}

#[test]
fn test_recording_toggle_without_pointer_stays_idle() {
    let mut orchestrator = create_test_orchestrator();
    let _ = orchestrator.update(OrchestratorMessage::GlobalHotkey(
        GlobalHotkeyAction::ToggleRecording,
    ));
    assert!(!orchestrator.is_recording());
}

#[test]
fn test_window_title_defaults_to_application_title() {
    let orchestrator = create_test_orchestrator();
    assert_eq!(
        orchestrator.window_title(Id::unique()),
        global_constants::APPLICATION_TITLE
    );
}
