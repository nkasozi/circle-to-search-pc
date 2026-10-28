#[cfg(target_os = "macos")]
mod macos_clipboard;

#[cfg(target_os = "macos")]
use macos_clipboard::{copy_image_with_platform_clipboard, copy_text_with_platform_clipboard};

const LOG_TAG: &str = "[CLIPBOARD]";
const CLIPBOARD_TEXT_COPY_STARTED_EVENT: &str = "clipboard_text_copy_started";
const CLIPBOARD_IMAGE_COPY_STARTED_EVENT: &str = "clipboard_image_copy_started";
#[cfg(not(target_os = "macos"))]
const CLIPBOARD_TEXT_COPY_SUCCEEDED_EVENT: &str = "clipboard_text_copy_succeeded";
#[cfg(not(target_os = "macos"))]
const CLIPBOARD_TEXT_COPY_FAILED_EVENT: &str = "clipboard_text_copy_failed";
#[cfg(not(target_os = "macos"))]
const CLIPBOARD_ACCESS_FAILED_EVENT: &str = "clipboard_access_failed";
#[cfg(not(target_os = "macos"))]
const CLIPBOARD_IMAGE_COPY_FAILED_EVENT: &str = "clipboard_image_copy_failed";
#[cfg(not(target_os = "macos"))]
const CLIPBOARD_IMAGE_COPY_SUCCEEDED_EVENT: &str = "clipboard_image_copy_succeeded";

pub fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": CLIPBOARD_TEXT_COPY_STARTED_EVENT, "character_count": text.len()})
    );
    copy_text_with_platform_clipboard(text)
}

pub fn copy_image_to_clipboard(rgba_data: &[u8], width: u32, height: u32) -> Result<(), String> {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": CLIPBOARD_IMAGE_COPY_STARTED_EVENT, "width": width, "height": height})
    );
    copy_image_with_platform_clipboard(rgba_data, width, height)
}

#[cfg(not(target_os = "macos"))]
fn copy_text_with_platform_clipboard(text: &str) -> Result<(), String> {
    match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text)) {
        Ok(()) => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_TEXT_COPY_SUCCEEDED_EVENT})
            );
            Ok(())
        }
        Err(error) => {
            let error_message = format!("Failed to copy to clipboard: {}", error);
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": CLIPBOARD_TEXT_COPY_FAILED_EVENT, "error_message": error_message})
            );
            Err(error_message)
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn copy_image_with_platform_clipboard(
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| {
        let error_message = format!("Failed to access clipboard: {}", error);
        log::error!("{} {}", LOG_TAG, serde_json::json!({"event": CLIPBOARD_ACCESS_FAILED_EVENT, "error_message": error_message}));
        error_message
    })?;
    let image_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: rgba_data.into(),
    };
    clipboard.set_image(image_data).map_err(|error| {
        let error_message = format!("Failed to copy image: {}", error);
        log::error!("{} {}", LOG_TAG, serde_json::json!({"event": CLIPBOARD_IMAGE_COPY_FAILED_EVENT, "error_message": error_message}));
        error_message
    })?;
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": CLIPBOARD_IMAGE_COPY_SUCCEEDED_EVENT})
    );
    Ok(())
}
