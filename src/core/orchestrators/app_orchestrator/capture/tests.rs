use super::error_messages::build_capture_error_message_for_platform;

#[test]
fn test_build_capture_error_message_linux_permission_error() {
    let error = "Access denied to screen capture";
    let result = build_capture_error_message_for_platform(error, "linux");
    assert!(result.contains("Screen capture failed"));
    assert!(result.contains("PipeWire"));
    assert!(result.contains("xdg-desktop-portal"));
    assert!(result.contains("sudo apt install"));
}

#[test]
fn test_build_capture_error_message_linux_pipewire_error() {
    let error = "PipeWire connection failed";
    let result = build_capture_error_message_for_platform(error, "linux");
    assert!(result.contains("Screen capture failed"));
    assert!(result.contains("PipeWire"));
}

#[test]
fn test_build_capture_error_message_linux_portal_error() {
    let error = "Portal request denied";
    let result = build_capture_error_message_for_platform(error, "linux");
    assert!(result.contains("Screen capture failed"));
    assert!(result.contains("xdg-desktop-portal"));
}

#[test]
fn test_build_capture_error_message_linux_generic_error() {
    let error = "Unknown capture error";
    let result = build_capture_error_message_for_platform(error, "linux");
    assert!(result.contains("Capture failed"));
    assert!(result.contains("Try closing other instances"));
    assert!(!result.contains("PipeWire"));
}

#[test]
fn test_build_capture_error_message_macos_permission_error() {
    let error = "Permission denied for screen recording";
    let result = build_capture_error_message_for_platform(error, "macos");
    assert!(result.contains("Screen capture failed"));
    assert!(result.contains("Screen Recording permission"));
    assert!(result.contains("System Settings"));
}

#[test]
fn test_build_capture_error_message_macos_access_error() {
    let error = "Access to screen not granted";
    let result = build_capture_error_message_for_platform(error, "macos");
    assert!(result.contains("Screen capture failed"));
    assert!(result.contains("Privacy & Security"));
}

#[test]
fn test_build_capture_error_message_macos_generic_error() {
    let error = "Monitor not found";
    let result = build_capture_error_message_for_platform(error, "macos");
    assert!(result.contains("Capture failed"));
    assert!(result.contains("Try closing other instances"));
    assert!(!result.contains("System Settings"));
}

#[test]
fn test_build_capture_error_message_windows_always_generic() {
    let error = "Permission denied";
    let result = build_capture_error_message_for_platform(error, "windows");
    assert!(result.contains("Capture failed"));
    assert!(result.contains("Try closing other instances"));
    assert!(!result.contains("PipeWire"));
    assert!(!result.contains("System Settings"));
}

#[test]
fn test_build_capture_error_message_unknown_platform() {
    let error = "Some error";
    let result = build_capture_error_message_for_platform(error, "freebsd");
    assert!(result.contains("Capture failed"));
    assert!(result.contains("Try closing other instances"));
}
