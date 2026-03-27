use std::cmp::Ordering;
use std::path::Path;

use crate::core::models::OcrResult;

use super::{CharPosition, ImageCopyState, OcrState, SaveState, SearchState};

const STATUS_PREPARING_SAVE_IMAGE: &str = "⏳ Preparing to save image...";
const STATUS_SAVING_IMAGE_FILE: &str = "💾 Saving image to file...";
const STATUS_PREPARING_IMAGE: &str = "⏳ Preparing image...";
const STATUS_COPYING_TO_CLIPBOARD: &str = "📋 Copying to clipboard...";
const STATUS_IMAGE_COPIED_TO_CLIPBOARD: &str = "✅ Image copied to clipboard";
const STATUS_UPLOADING_IMAGE_FOR_SEARCH: &str = "🔍 Uploading image for search...";
const STATUS_SEARCH_COMPLETED: &str = "✅ Search completed";
const STATUS_DRAW_MODE_ENABLED: &str = "🖊️ Draw Mode ON - Click and drag to draw";
const STATUS_PROMPT_PERFORM_OCR: &str = "Perform OCR text recognition?";
const STATUS_PROCESSING_OCR: &str = "Processing OCR...";
const STATUS_SAVE_SUCCESS_PREFIX: &str = "✅ Saved to ";
const STATUS_SAVE_FAILED_PREFIX: &str = "❌ Save failed: ";
const STATUS_COPY_FAILED_PREFIX: &str = "❌ Copy failed: ";
const STATUS_SEARCH_FAILED_PREFIX: &str = "❌ Search failed: ";
const STATUS_DETECTED_WORDS_PREFIX: &str = "✅ Detected ";
const STATUS_DETECTED_WORDS_SUFFIX: &str = " words - Click to select text";
const STATUS_SELECTED_CHARACTERS_PREFIX: &str = "Selected ";
const STATUS_SELECTED_CHARACTERS_SUFFIX: &str = " characters";

pub fn build_selected_text_with_layout(
    selected_chars: &[usize],
    char_positions: &[CharPosition],
) -> String {
    if selected_chars.is_empty() {
        return String::new();
    }

    let mut selected_positions: Vec<&CharPosition> = selected_chars
        .iter()
        .filter_map(|&index| char_positions.get(index))
        .collect();

    selected_positions.sort_by(compare_char_positions);

    let Some(first_position) = selected_positions.first().copied() else {
        return String::new();
    };

    let mut result = String::new();
    let mut last_y = first_position.bounds.y;
    let mut last_word_index = first_position.word_index;
    let mut last_x_end = first_position.bounds.x + first_position.bounds.width;

    for position in selected_positions {
        let line_height_threshold = position.bounds.height * 0.5;
        let y_difference = (position.bounds.y - last_y).abs();

        let position_has_invalid_coords =
            !position.bounds.x.is_finite() || !position.bounds.y.is_finite();
        let last_has_invalid_coords = !last_x_end.is_finite() || !last_y.is_finite();

        let should_add_newline = !position_has_invalid_coords
            && !last_has_invalid_coords
            && y_difference > line_height_threshold;

        if should_add_newline {
            result.push('\n');
            last_y = position.bounds.y;
            last_word_index = position.word_index;
            last_x_end = position.bounds.x + position.bounds.width;
        } else if position.word_index != last_word_index {
            let gap_between_words = position.bounds.x - last_x_end;
            let space_threshold = position.bounds.width * 0.3;
            if gap_between_words > space_threshold {
                result.push(' ');
            }
            last_word_index = position.word_index;
            last_x_end = position.bounds.x + position.bounds.width;
        } else {
            last_x_end = position.bounds.x + position.bounds.width;
        }

        result.push(position.character);
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
    match (save_state, image_copy_state, search_state, ocr_state) {
        (SaveState::Preparing, _, _, _) => STATUS_PREPARING_SAVE_IMAGE.to_string(),
        (SaveState::Saving, _, _, _) => STATUS_SAVING_IMAGE_FILE.to_string(),
        (SaveState::Success(path), _, _, _) => {
            format!(
                "{}{}",
                STATUS_SAVE_SUCCESS_PREFIX,
                extract_display_name(path)
            )
        }
        (SaveState::Failed(error_message), _, _, _) => {
            format!("{}{}", STATUS_SAVE_FAILED_PREFIX, error_message)
        }
        (_, ImageCopyState::Preparing, _, _) => STATUS_PREPARING_IMAGE.to_string(),
        (_, ImageCopyState::Copying, _, _) => STATUS_COPYING_TO_CLIPBOARD.to_string(),
        (_, ImageCopyState::Success, _, _) => STATUS_IMAGE_COPIED_TO_CLIPBOARD.to_string(),
        (_, ImageCopyState::Failed(error_message), _, _) => {
            format!("{}{}", STATUS_COPY_FAILED_PREFIX, error_message)
        }
        (_, _, SearchState::UploadingImage, _) => STATUS_UPLOADING_IMAGE_FOR_SEARCH.to_string(),
        (_, _, SearchState::Completed, _) => STATUS_SEARCH_COMPLETED.to_string(),
        (_, _, SearchState::Failed(error_message), _) => {
            format!("{}{}", STATUS_SEARCH_FAILED_PREFIX, error_message)
        }
        (_, _, _, _) if draw_mode_enabled => STATUS_DRAW_MODE_ENABLED.to_string(),
        (_, _, _, OcrState::Idle) => STATUS_PROMPT_PERFORM_OCR.to_string(),
        (_, _, _, OcrState::Processing) => STATUS_PROCESSING_OCR.to_string(),
        (_, _, _, OcrState::Failed(_)) => String::new(),
        (_, _, _, OcrState::Completed) => match ocr_result {
            Some(result) if selected_char_count == 0 => format!(
                "{}{}{}",
                STATUS_DETECTED_WORDS_PREFIX,
                result.text_blocks.len(),
                STATUS_DETECTED_WORDS_SUFFIX,
            ),
            Some(_) => format!(
                "{}{}{}",
                STATUS_SELECTED_CHARACTERS_PREFIX,
                selected_char_count,
                STATUS_SELECTED_CHARACTERS_SUFFIX,
            ),
            None => String::new(),
        },
    }
}

fn compare_char_positions(left: &&CharPosition, right: &&CharPosition) -> Ordering {
    let left_y_valid = left.bounds.y.is_finite();
    let right_y_valid = right.bounds.y.is_finite();

    let on_same_line = if left_y_valid && right_y_valid {
        let y_difference = (left.bounds.y - right.bounds.y).abs();
        let line_height_threshold = left.bounds.height * 0.5;
        y_difference <= line_height_threshold
    } else if !left_y_valid || !right_y_valid {
        true
    } else {
        false
    };

    if on_same_line {
        let left_x = if left.bounds.x.is_finite() {
            left.bounds.x
        } else {
            f32::INFINITY
        };
        let right_x = if right.bounds.x.is_finite() {
            right.bounds.x
        } else {
            f32::INFINITY
        };
        left_x.total_cmp(&right_x)
    } else {
        let left_y = if left.bounds.y.is_finite() {
            left.bounds.y
        } else {
            f32::INFINITY
        };
        let right_y = if right.bounds.y.is_finite() {
            right.bounds.y
        } else {
            f32::INFINITY
        };
        left_y.total_cmp(&right_y)
    }
}

fn extract_display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string())
}

#[cfg(test)]
mod tests {
    use iced::Rectangle;

    use crate::core::models::OcrResult;

    use super::*;

    fn build_position(
        word_index: usize,
        char_index: usize,
        x: f32,
        y: f32,
        character: char,
    ) -> CharPosition {
        CharPosition {
            word_index,
            char_index,
            bounds: Rectangle {
                x,
                y,
                width: 10.0,
                height: 20.0,
            },
            character,
        }
    }

    #[test]
    fn test_build_selected_text_with_layout_sorts_nan_coordinates_without_panicking() {
        let char_positions = vec![
            build_position(0, 0, f32::NAN, 30.0, 'B'),
            build_position(0, 1, 10.0, 10.0, 'A'),
        ];

        let result = build_selected_text_with_layout(&[0, 1], &char_positions);

        assert_eq!(result, "AB");
    }

    #[test]
    fn test_build_selected_text_with_layout_inserts_spaces_between_words() {
        let char_positions = vec![
            build_position(0, 0, 0.0, 10.0, 'H'),
            build_position(0, 1, 10.0, 10.0, 'i'),
            build_position(1, 0, 30.0, 10.0, 'T'),
            build_position(1, 1, 40.0, 10.0, 'h'),
        ];

        let result = build_selected_text_with_layout(&[0, 1, 2, 3], &char_positions);

        assert_eq!(result, "Hi Th");
    }

    #[test]
    fn test_build_status_text_uses_file_name_for_save_success() {
        let result = build_status_text(
            &SaveState::Success("/tmp/capture.png".to_string()),
            &ImageCopyState::Idle,
            &SearchState::Idle,
            &OcrState::Completed,
            false,
            None,
            0,
        );

        assert_eq!(result, "✅ Saved to capture.png");
    }

    #[test]
    fn test_build_status_text_reports_detected_word_count() {
        let result = build_status_text(
            &SaveState::Idle,
            &ImageCopyState::Idle,
            &SearchState::Idle,
            &OcrState::Completed,
            false,
            Some(&OcrResult {
                text_blocks: vec![],
                full_text: String::new(),
            }),
            0,
        );

        assert_eq!(result, "✅ Detected 0 words - Click to select text");
    }
}
