use super::ocr_replacements::get_ocr_replacements;
use super::text_cleanup::{cleanup_ocr_artifacts, parse_bounds_from_debug};
use super::{TesseractOcrService, TRAINING_DATA, TRAINING_DATA_FILE_NAME};
use std::path::PathBuf;
use uuid::Uuid;

fn build_test_tessdata_dir() -> PathBuf {
    std::env::temp_dir().join(format!("circle-to-search-tessdata-test-{}", Uuid::new_v4()))
}

#[test]
fn test_build_creates_service_successfully() {
    let result = TesseractOcrService::build();

    assert!(result.is_ok());
}

#[test]
fn test_ensure_tessdata_ready_recreates_missing_training_data_file() {
    let tessdata_dir = build_test_tessdata_dir();

    TesseractOcrService::ensure_tessdata_ready_at_path(&tessdata_dir).unwrap();

    let eng_traineddata_path = tessdata_dir.join(TRAINING_DATA_FILE_NAME);
    std::fs::remove_file(&eng_traineddata_path).unwrap();

    TesseractOcrService::ensure_tessdata_ready_at_path(&tessdata_dir).unwrap();

    assert!(eng_traineddata_path.exists());
    assert_eq!(
        std::fs::metadata(&eng_traineddata_path).unwrap().len(),
        TRAINING_DATA.len() as u64
    );

    std::fs::remove_dir_all(&tessdata_dir).unwrap();
}

#[test]
fn test_cleanup_ocr_artifacts_removes_garbage_characters() {
    assert_eq!(cleanup_ocr_artifacts("¬Æ test"), "test");
    assert_eq!(cleanup_ocr_artifacts("¬© text"), "text");
    assert_eq!(cleanup_ocr_artifacts("hello (|) world"), "hello  world");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_bullets() {
    assert_eq!(cleanup_ocr_artifacts("¬¢ item"), "• item");
    assert_eq!(cleanup_ocr_artifacts("¬∑ bullet"), "• bullet");
    assert_eq!(cleanup_ocr_artifacts("¬∏ point"), "• point");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_quotes() {
    assert_eq!(cleanup_ocr_artifacts("¬´hello¬ª"), "'hello'");
    assert_eq!(cleanup_ocr_artifacts("``quoted''"), "\"quoted\"");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_arrows() {
    assert_eq!(cleanup_ocr_artifacts("go ->"), "go →");
    assert_eq!(cleanup_ocr_artifacts("<- back"), "← back");
    assert_eq!(cleanup_ocr_artifacts("<->"), "↔");
    assert_eq!(cleanup_ocr_artifacts("=>"), "⇒");
    assert_eq!(cleanup_ocr_artifacts("t/\\"), "↑/↓");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_mac_shortcuts() {
    assert_eq!(cleanup_ocr_artifacts("#C"), "⌘C");
    assert_eq!(cleanup_ocr_artifacts("#V"), "⌘V");
    assert_eq!(cleanup_ocr_artifacts("#S"), "⌘S");
    assert_eq!(cleanup_ocr_artifacts("#Z"), "⌘Z");
    assert_eq!(
        cleanup_ocr_artifacts("Press #C to copy"),
        "Press ⌘C to copy"
    );
}

#[test]
fn test_cleanup_ocr_artifacts_converts_modifier_keys() {
    assert_eq!(cleanup_ocr_artifacts("Ctrl+C"), "⌃C");
    assert_eq!(cleanup_ocr_artifacts("Alt+Tab"), "⌥⇥");
    assert_eq!(cleanup_ocr_artifacts("Shift+Enter"), "⇧↵");
    assert_eq!(cleanup_ocr_artifacts("Cmd+Q"), "⌘Q");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_special_keys() {
    assert_eq!(cleanup_ocr_artifacts("Tab"), "⇥");
    assert_eq!(cleanup_ocr_artifacts("Enter"), "↵");
    assert_eq!(cleanup_ocr_artifacts("Esc"), "⎋");
    assert_eq!(cleanup_ocr_artifacts("Del"), "⌫");
    assert_eq!(cleanup_ocr_artifacts("Backspace"), "⌫");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_math_symbols() {
    assert_eq!(cleanup_ocr_artifacts("+-"), "±");
    assert_eq!(cleanup_ocr_artifacts("!="), "≠");
    assert_eq!(cleanup_ocr_artifacts("..."), "…");
    assert_eq!(cleanup_ocr_artifacts("1/2"), "½");
    assert_eq!(cleanup_ocr_artifacts("pi"), "π");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_copyright_symbols() {
    assert_eq!(cleanup_ocr_artifacts("(c)"), "©");
    assert_eq!(cleanup_ocr_artifacts("(C)"), "©");
    assert_eq!(cleanup_ocr_artifacts("(r)"), "®");
    assert_eq!(cleanup_ocr_artifacts("(tm)"), "™");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_checkboxes() {
    assert_eq!(cleanup_ocr_artifacts("[x]"), "☑");
    assert_eq!(cleanup_ocr_artifacts("[ ]"), "☐");
    assert_eq!(cleanup_ocr_artifacts("[v]"), "✓");
}

#[test]
fn test_cleanup_ocr_artifacts_converts_currency() {
    assert_eq!(cleanup_ocr_artifacts("EUR 100"), "€ 100");
    assert_eq!(cleanup_ocr_artifacts("GBP 50"), "£ 50");
    assert_eq!(cleanup_ocr_artifacts("JPY 1000"), "¥ 1000");
}

#[test]
fn test_cleanup_ocr_artifacts_trims_whitespace() {
    assert_eq!(cleanup_ocr_artifacts("  hello  "), "hello");
    assert_eq!(cleanup_ocr_artifacts("\ttest\n"), "test");
}

#[test]
fn test_cleanup_ocr_artifacts_handles_empty_string() {
    assert_eq!(cleanup_ocr_artifacts(""), "");
}

#[test]
fn test_cleanup_ocr_artifacts_preserves_normal_text() {
    assert_eq!(cleanup_ocr_artifacts("Hello World"), "Hello World");
    assert_eq!(cleanup_ocr_artifacts("Test 123"), "Test 123");
}

#[test]
fn test_cleanup_ocr_artifacts_handles_complex_text() {
    let input = "Press #C to copy ¬¢ item -> next";
    let expected = "Press ⌘C to copy • item → next";
    assert_eq!(cleanup_ocr_artifacts(input), expected);
}

#[test]
fn test_parse_bounds_from_debug_extracts_coordinates() {
    let result = parse_bounds_from_debug("Rect { min: (10, 20), max: (100, 50) }");
    assert_eq!(result, (10.0, 20.0, 90.0, 30.0));
}

#[test]
fn test_parse_bounds_from_debug_handles_invalid_input() {
    let result = parse_bounds_from_debug("invalid");
    assert_eq!(result, (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_get_ocr_replacements_returns_non_empty_list() {
    let replacements = get_ocr_replacements();
    assert!(!replacements.is_empty());
    assert!(replacements.len() > 100);
}

#[test]
fn test_get_ocr_replacements_contains_essential_patterns() {
    let replacements = get_ocr_replacements();
    let patterns: Vec<&str> = replacements.iter().map(|(p, _)| *p).collect();

    assert!(patterns.contains(&"#C"));
    assert!(patterns.contains(&"#V"));
    assert!(patterns.contains(&"->"));
    assert!(patterns.contains(&"..."));
    assert!(patterns.contains(&"(c)"));
}
