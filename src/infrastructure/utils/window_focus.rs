#[cfg(target_os = "macos")]
const OSASCRIPT_COMMAND: &str = "osascript";
#[cfg(target_os = "macos")]
const OSASCRIPT_EXPRESSION_FLAG: &str = "-e";
#[cfg(not(target_os = "macos"))]
const WINDOW_FOCUS_NOT_SUPPORTED: &str = "Window focus not supported on this platform";
const LOG_TAG: &str = "[WINDOW_FOCUS]";
const WINDOW_FOCUS_REQUESTED_EVENT: &str = "window_focus_requested";
#[cfg(target_os = "macos")]
const WINDOW_FOCUS_SUCCEEDED_EVENT: &str = "window_focus_succeeded";
#[cfg(target_os = "macos")]
const WINDOW_FOCUS_SCRIPT_FAILED_EVENT: &str = "window_focus_script_failed";
#[cfg(target_os = "macos")]
const WINDOW_FOCUS_COMMAND_FAILED_EVENT: &str = "window_focus_command_failed";
#[cfg(not(target_os = "macos"))]
const WINDOW_FOCUS_UNSUPPORTED_EVENT: &str = "window_focus_unsupported";

pub fn focus_external_window_by_app_name(app_name: &str) -> Result<(), String> {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": WINDOW_FOCUS_REQUESTED_EVENT, "app_name": app_name})
    );
    focus_window_with_platform_api(app_name)
}

#[cfg(target_os = "macos")]
fn focus_window_with_platform_api(app_name: &str) -> Result<(), String> {
    use std::process::Command;

    let script = format!(
        r#"tell application "System Events"
                set targetProcess to first process whose name is "{}"
                set frontmost of targetProcess to true
            end tell"#,
        app_name
    );

    let result = Command::new(OSASCRIPT_COMMAND)
        .arg(OSASCRIPT_EXPRESSION_FLAG)
        .arg(&script)
        .output();

    interpret_focus_result(app_name, result)
}

#[cfg(target_os = "macos")]
fn interpret_focus_result(
    app_name: &str,
    result: std::io::Result<std::process::Output>,
) -> Result<(), String> {
    match result {
        Ok(output) if output.status.success() => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": WINDOW_FOCUS_SUCCEEDED_EVENT, "app_name": app_name})
            );
            Ok(())
        }
        Ok(output) => {
            let standard_error = String::from_utf8_lossy(&output.stderr);
            let error_message = format!("AppleScript failed: {}", standard_error);
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": WINDOW_FOCUS_SCRIPT_FAILED_EVENT, "app_name": app_name, "error_message": error_message})
            );
            Err(error_message)
        }
        Err(error) => {
            let error_message = format!("Failed to run osascript: {}", error);
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": WINDOW_FOCUS_COMMAND_FAILED_EVENT, "app_name": app_name, "error_message": error_message})
            );
            Err(error_message)
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn focus_window_with_platform_api(_app_name: &str) -> Result<(), String> {
    log::warn!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": WINDOW_FOCUS_UNSUPPORTED_EVENT})
    );
    Err(WINDOW_FOCUS_NOT_SUPPORTED.to_string())
}
