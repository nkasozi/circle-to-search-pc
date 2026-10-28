use std::path::{Path, PathBuf};

const SCREENSHOT_FILENAME_PREFIX: &str = "screenshot_";
const SCREENSHOT_FILENAME_SUFFIX: &str = ".png";
const LOG_TAG_FILE_SAVE: &str = "[FILE_SAVE]";
const LOG_TAG_IMAGE_CONVERT: &str = "[IMAGE_CONVERT]";
const IMAGE_FILE_SAVE_STARTED_EVENT: &str = "image_file_save_started";
const IMAGE_FILE_SAVE_SUCCEEDED_EVENT: &str = "image_file_save_succeeded";
const SAVE_TIMESTAMP_CALCULATION_FAILED_EVENT: &str = "save_timestamp_calculation_failed";
const SAVE_DIRECTORY_CREATION_FAILED_EVENT: &str = "save_directory_creation_failed";
const IMAGE_FILE_WRITE_FAILED_EVENT: &str = "image_file_write_failed";
const RGBA_IMAGE_CREATION_FAILED_EVENT: &str = "rgba_image_creation_failed";
const PNG_ENCODING_FAILED_EVENT: &str = "png_encoding_failed";

pub fn save_image_to_file(
    rgba_data: &[u8],
    width: u32,
    height: u32,
    save_location: &str,
) -> Result<String, String> {
    log::info!(
        "{} {}",
        LOG_TAG_FILE_SAVE,
        serde_json::json!({"event": IMAGE_FILE_SAVE_STARTED_EVENT, "width": width, "height": height, "save_location": save_location})
    );

    let save_path = build_screenshot_save_path(save_location)?;
    ensure_parent_directory_exists(&save_path)?;
    let png_data = convert_rgba_to_png(rgba_data, width, height)?;
    write_png_file(&save_path, png_data)?;

    let saved_path_text = save_path.to_string_lossy().to_string();
    log::info!(
        "{} {}",
        LOG_TAG_FILE_SAVE,
        serde_json::json!({"event": IMAGE_FILE_SAVE_SUCCEEDED_EVENT, "saved_path": saved_path_text})
    );
    Ok(saved_path_text)
}

fn build_screenshot_save_path(save_location: &str) -> Result<PathBuf, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| {
            let error_message = format!("Failed to calculate timestamp: {}", error);
            log::error!("{} {}", LOG_TAG_FILE_SAVE, serde_json::json!({"event": SAVE_TIMESTAMP_CALCULATION_FAILED_EVENT, "error_message": error_message}));
            error_message
        })?
        .as_secs();
    let filename = format!(
        "{}{}{}",
        SCREENSHOT_FILENAME_PREFIX, timestamp, SCREENSHOT_FILENAME_SUFFIX
    );
    Ok(PathBuf::from(save_location).join(&filename))
}

fn ensure_parent_directory_exists(save_path: &Path) -> Result<bool, String> {
    let Some(parent) = save_path.parent() else {
        return Ok(false);
    };
    std::fs::create_dir_all(parent).map_err(|error| {
        let error_message = format!("Failed to create directory: {}", error);
        log::error!("{} {}", LOG_TAG_FILE_SAVE, serde_json::json!({"event": SAVE_DIRECTORY_CREATION_FAILED_EVENT, "error_message": error_message}));
        error_message
    })?;
    Ok(true)
}

fn write_png_file(save_path: &Path, png_data: Vec<u8>) -> Result<bool, String> {
    std::fs::write(save_path, png_data).map_err(|error| {
        let error_message = format!("Failed to write file: {}", error);
        log::error!("{} {}", LOG_TAG_FILE_SAVE, serde_json::json!({"event": IMAGE_FILE_WRITE_FAILED_EVENT, "error_message": error_message}));
        error_message
    })?;
    Ok(true)
}

pub(super) fn convert_rgba_to_png(
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let rgba_image =
        image::RgbaImage::from_raw(width, height, rgba_data.to_vec()).ok_or_else(|| {
            let error_message = crate::global_constants::OCR_RAW_IMAGE_CREATION_FAILED.to_string();
            log::error!("{} {}", LOG_TAG_IMAGE_CONVERT, serde_json::json!({"event": RGBA_IMAGE_CREATION_FAILED_EVENT, "error_message": error_message}));
            error_message
        })?;

    let mut png_data = Vec::new();
    image::DynamicImage::ImageRgba8(rgba_image)
        .write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )
        .map_err(|error| {
            let error_message = format!("Failed to encode PNG: {}", error);
            log::error!("{} {}", LOG_TAG_IMAGE_CONVERT, serde_json::json!({"event": PNG_ENCODING_FAILED_EVENT, "error_message": error_message}));
            error_message
        })?;

    Ok(png_data)
}
