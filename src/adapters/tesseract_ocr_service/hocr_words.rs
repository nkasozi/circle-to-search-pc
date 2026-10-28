use tesseract_static::parse::ParsedHocr;

use super::text_cleanup::{cleanup_ocr_artifacts, parse_bounds_from_debug};
use crate::core::models::{DetectedText, DetectedWord, OcrResult};

const MINIMUM_WORD_CONFIDENCE: f32 = 0.30;
const CONFIDENCE_PERCENT_MULTIPLIER: f32 = 100.0;
const LOG_TAG: &str = "[TESSERACT_OCR]";
const TEXT_EXTRACTION_COMPLETED_EVENT: &str = "text_extraction_completed";
const TEXT_EXTRACTION_FULL_TEXT_EVENT: &str = "text_extraction_full_text";
const WORD_DETECTED_EVENT: &str = "word_detected";
const LOW_CONFIDENCE_WORD_SKIPPED_EVENT: &str = "low_confidence_word_skipped";

pub(super) fn build_ocr_result_from_hocr(hocr: &ParsedHocr) -> OcrResult {
    let detected_texts: Vec<DetectedText> = hocr
        .careas
        .iter()
        .flat_map(|content_area| content_area.paragraphs.iter())
        .flat_map(|paragraph| paragraph.lines.iter())
        .flat_map(|line| line.words.iter())
        .filter_map(|word| build_detected_text_from_word(&word.text, word.confidence, &word.bounds))
        .collect();

    let full_text = build_full_text(&detected_texts);

    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": TEXT_EXTRACTION_COMPLETED_EVENT, "word_count": detected_texts.len()})
    );
    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": TEXT_EXTRACTION_FULL_TEXT_EVENT, "full_text": full_text})
    );

    OcrResult {
        text_blocks: detected_texts,
        full_text,
    }
}

fn build_full_text(detected_texts: &[DetectedText]) -> String {
    let full_text =
        detected_texts
            .iter()
            .fold(String::new(), |mut accumulated_text, detected_text| {
                accumulated_text.push_str(&detected_text.content);
                accumulated_text.push(' ');
                accumulated_text
            });
    full_text.trim().to_string()
}

fn build_detected_text_from_word(
    raw_text: &str,
    confidence: f32,
    bounds: &impl std::fmt::Debug,
) -> Option<DetectedText> {
    let word_text = accept_word_text(raw_text, confidence)?;

    let word_bounds_text = format!("{:?}", bounds);
    let (x, y, width, height) = parse_bounds_from_debug(&word_bounds_text);

    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": WORD_DETECTED_EVENT, "word_text": word_text, "x_position": x, "y_position": y, "width": width, "height": height, "confidence_percent": confidence * CONFIDENCE_PERCENT_MULTIPLIER})
    );

    let word = DetectedWord::new(word_text.clone(), x, y, width, height);
    Some(DetectedText::new(
        word_text,
        x,
        y,
        width,
        height,
        confidence,
        vec![word],
    ))
}

fn accept_word_text(raw_text: &str, confidence: f32) -> Option<String> {
    let raw_word_text = raw_text.trim();
    if raw_word_text.is_empty() {
        return None;
    }

    if confidence < MINIMUM_WORD_CONFIDENCE {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": LOW_CONFIDENCE_WORD_SKIPPED_EVENT, "word_text": raw_word_text, "confidence_percent": confidence * CONFIDENCE_PERCENT_MULTIPLIER})
        );
        return None;
    }

    let word_text = cleanup_ocr_artifacts(raw_word_text);
    match word_text.is_empty() {
        true => None,
        false => Some(word_text),
    }
}
