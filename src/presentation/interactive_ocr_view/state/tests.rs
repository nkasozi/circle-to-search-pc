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

    assert_eq!(result, "Saved to capture.png");
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

    assert_eq!(result, "Found 0 words — drag to select text");
}
