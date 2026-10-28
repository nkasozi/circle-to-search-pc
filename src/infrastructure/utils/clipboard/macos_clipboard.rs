use std::io::Write;
use std::process::{Command, Output, Stdio};

use crate::infrastructure::utils::image_files::convert_rgba_to_png;

const CLIPBOARD_TEMP_IMAGE_FILENAME: &str = "circle_to_search_clipboard.png";
const CLIPBOARD_TEMP_PATH_INVALID: &str = "Invalid temp path";
const PBCOPY_COMMAND: &str = "pbcopy";
const OSASCRIPT_COMMAND: &str = "osascript";
const OSASCRIPT_EXPRESSION_FLAG: &str = "-e";
const LOG_TAG: &str = "[CLIPBOARD]";
const CLIPBOARD_TEXT_COPY_SUCCEEDED_EVENT: &str = "clipboard_text_copy_succeeded";
const CLIPBOARD_TEXT_COPY_FAILED_EVENT: &str = "clipboard_text_copy_failed";
const CLIPBOARD_IMAGE_COPY_SUCCEEDED_EVENT: &str = "clipboard_image_copy_succeeded";
const CLIPBOARD_IMAGE_COPY_FAILED_EVENT: &str = "clipboard_image_copy_failed";

pub(super) fn copy_text_with_platform_clipboard(text: &str) -> Result<(), String> {
    let result = Command::new(PBCOPY_COMMAND)
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()
        });

    match result {
        Ok(status) if status.success() => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_TEXT_COPY_SUCCEEDED_EVENT, "clipboard_backend": PBCOPY_COMMAND})
            );
            Ok(())
        }
        Ok(status) => {
            let error_message = format!("pbcopy exited with status: {:?}", status.code());
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_TEXT_COPY_FAILED_EVENT, "error_message": error_message})
            );
            Err(error_message)
        }
        Err(error) => {
            let error_message = format!("Failed to run pbcopy: {}", error);
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_TEXT_COPY_FAILED_EVENT, "error_message": error_message})
            );
            Err(error_message)
        }
    }
}

pub(super) fn copy_image_with_platform_clipboard(
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let png_data = convert_rgba_to_png(rgba_data, width, height)?;

    let temporary_path = std::env::temp_dir().join(CLIPBOARD_TEMP_IMAGE_FILENAME);
    std::fs::write(&temporary_path, &png_data)
        .map_err(|error| format!("Failed to write temp file: {}", error))?;

    let temporary_path_text = temporary_path.to_str().ok_or(CLIPBOARD_TEMP_PATH_INVALID)?;

    let script = format!(
        "set the clipboard to (read (POSIX file \"{}\") as «class PNGf»)",
        temporary_path_text
    );

    let result = Command::new(OSASCRIPT_COMMAND)
        .arg(OSASCRIPT_EXPRESSION_FLAG)
        .arg(&script)
        .output();

    let _ = std::fs::remove_file(&temporary_path);

    interpret_osascript_result(result)
}

fn interpret_osascript_result(result: std::io::Result<Output>) -> Result<(), String> {
    match result {
        Ok(output) if output.status.success() => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_IMAGE_COPY_SUCCEEDED_EVENT, "clipboard_backend": OSASCRIPT_COMMAND})
            );
            Ok(())
        }
        Ok(output) => {
            let standard_error = String::from_utf8_lossy(&output.stderr);
            let error_message = format!(
                "osascript exited with status: {:?}, stderr: {}",
                output.status.code(),
                standard_error.trim()
            );
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_IMAGE_COPY_FAILED_EVENT, "error_message": error_message})
            );
            Err(error_message)
        }
        Err(error) => {
            let error_message = format!("Failed to run osascript: {}", error);
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_IMAGE_COPY_FAILED_EVENT, "error_message": error_message})
            );
            Err(error_message)
        }
    }
}
