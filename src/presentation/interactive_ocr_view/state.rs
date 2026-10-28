use std::cmp::Ordering;
use std::path::Path;

use crate::core::models::OcrResult;

use super::{CharPosition, ImageCopyState, OcrState, SaveState, SearchState};

mod layout_cursor;

#[cfg(test)]
mod tests;

use layout_cursor::{advance_layout_cursor, LayoutCursor, LINE_HEIGHT_THRESHOLD_RATIO};

const STATUS_PREPARING_SAVE_IMAGE: &str = "Preparing to save…";
const STATUS_SAVING_IMAGE_FILE: &str = "Saving image…";
const STATUS_PREPARING_IMAGE: &str = "Preparing image…";
const STATUS_COPYING_TO_CLIPBOARD: &str = "Copying…";
const STATUS_IMAGE_COPIED_TO_CLIPBOARD: &str = "Image copied";
const STATUS_UPLOADING_IMAGE_FOR_SEARCH: &str = "Searching…";
const STATUS_SEARCH_COMPLETED: &str = "Opened in your browser";
const STATUS_DRAW_MODE_ENABLED: &str = "Drawing — drag on the image";
const STATUS_PROMPT_PERFORM_OCR: &str = "Perform OCR text recognition?";
const STATUS_PROCESSING_OCR: &str = "Reading text…";
const STATUS_SAVE_SUCCESS_PREFIX: &str = "Saved to ";
const STATUS_SAVE_FAILED_PREFIX: &str = "Couldn't save: ";
const STATUS_COPY_FAILED_PREFIX: &str = "Couldn't copy: ";
const STATUS_SEARCH_FAILED_PREFIX: &str = "Search failed: ";
const STATUS_DETECTED_WORDS_PREFIX: &str = "Found ";
const STATUS_DETECTED_WORDS_SUFFIX: &str = " words — drag to select text";
const STATUS_SELECTED_CHARACTERS_PREFIX: &str = "Selected ";
const STATUS_SELECTED_CHARACTERS_SUFFIX: &str = " characters selected";

pub fn build_selected_text_with_layout(
    selected_chars: &[usize],
    char_positions: &[CharPosition],
) -> String {
    let mut selected_positions: Vec<&CharPosition> = selected_chars
        .iter()
        .filter_map(|&index| char_positions.get(index))
        .collect();

    selected_positions.sort_by(compare_char_positions);

    let Some(first_position) = selected_positions.first().copied() else {
        return String::new();
    };

    let mut result = String::new();
    let mut layout_cursor = LayoutCursor::starting_at(first_position);

    for position in selected_positions {
        let (separator, next_cursor) = advance_layout_cursor(&layout_cursor, position);
        result.extend(separator);
        result.push(position.character);
        layout_cursor = next_cursor;
    }

    result
}

pub fn build_status_text(
    save_state: &SaveState,
    image_copy_state: &ImageCopyState,
    search_state: &SearchState,
    ocr_state: &OcrState,
    draw_mode_enabled: bool,
    ocr_result: Option<&OcrResult>,
    selected_char_count: usize,
) -> String {
    save_status_text(save_state)
        .or_else(|| image_copy_status_text(image_copy_state))
        .or_else(|| search_status_text(search_state))
        .unwrap_or_else(|| match draw_mode_enabled {
            true => STATUS_DRAW_MODE_ENABLED.to_string(),
            false => ocr_status_text(ocr_state, ocr_result, selected_char_count),
        })
}

fn save_status_text(save_state: &SaveState) -> Option<String> {
    match save_state {
        SaveState::Idle => None,
        SaveState::Preparing => Some(STATUS_PREPARING_SAVE_IMAGE.to_string()),
        SaveState::Saving => Some(STATUS_SAVING_IMAGE_FILE.to_string()),
        SaveState::Success(path) => Some(format!(
            "{}{}",
            STATUS_SAVE_SUCCESS_PREFIX,
            extract_display_name(path)
        )),
        SaveState::Failed(error_message) => {
            Some(format!("{}{}", STATUS_SAVE_FAILED_PREFIX, error_message))
        }
    }
}

fn image_copy_status_text(image_copy_state: &ImageCopyState) -> Option<String> {
    match image_copy_state {
        ImageCopyState::Idle => None,
        ImageCopyState::Preparing => Some(STATUS_PREPARING_IMAGE.to_string()),
        ImageCopyState::Copying => Some(STATUS_COPYING_TO_CLIPBOARD.to_string()),
        ImageCopyState::Success => Some(STATUS_IMAGE_COPIED_TO_CLIPBOARD.to_string()),
        ImageCopyState::Failed(error_message) => {
            Some(format!("{}{}", STATUS_COPY_FAILED_PREFIX, error_message))
        }
    }
}

fn search_status_text(search_state: &SearchState) -> Option<String> {
    match search_state {
        SearchState::Idle => None,
        SearchState::UploadingImage => Some(STATUS_UPLOADING_IMAGE_FOR_SEARCH.to_string()),
        SearchState::Completed => Some(STATUS_SEARCH_COMPLETED.to_string()),
        SearchState::Failed(error_message) => {
            Some(format!("{}{}", STATUS_SEARCH_FAILED_PREFIX, error_message))
        }
    }
}

fn ocr_status_text(
    ocr_state: &OcrState,
    ocr_result: Option<&OcrResult>,
    selected_char_count: usize,
) -> String {
    match (ocr_state, ocr_result) {
        (OcrState::Idle, _) => STATUS_PROMPT_PERFORM_OCR.to_string(),
        (OcrState::Processing, _) => STATUS_PROCESSING_OCR.to_string(),
        (OcrState::Failed(_), _) | (OcrState::Completed, None) => String::new(),
        (OcrState::Completed, Some(result)) => match selected_char_count {
            0 => format!(
                "{}{}{}",
                STATUS_DETECTED_WORDS_PREFIX,
                result.text_blocks.len(),
                STATUS_DETECTED_WORDS_SUFFIX,
            ),
            _ => format!(
                "{}{}{}",
                STATUS_SELECTED_CHARACTERS_PREFIX,
                selected_char_count,
                STATUS_SELECTED_CHARACTERS_SUFFIX,
            ),
        },
    }
}

fn compare_char_positions(left: &&CharPosition, right: &&CharPosition) -> Ordering {
    let both_y_valid = left.bounds.y.is_finite() && right.bounds.y.is_finite();
    let on_same_line = match both_y_valid {
        true => {
            let y_difference = (left.bounds.y - right.bounds.y).abs();
            y_difference <= left.bounds.height * LINE_HEIGHT_THRESHOLD_RATIO
        }
        false => true,
    };

    match on_same_line {
        true => finite_or_infinity(left.bounds.x).total_cmp(&finite_or_infinity(right.bounds.x)),
        false => finite_or_infinity(left.bounds.y).total_cmp(&finite_or_infinity(right.bounds.y)),
    }
}

fn finite_or_infinity(value: f32) -> f32 {
    match value.is_finite() {
        true => value,
        false => f32::INFINITY,
    }
}

fn extract_display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string())
}
