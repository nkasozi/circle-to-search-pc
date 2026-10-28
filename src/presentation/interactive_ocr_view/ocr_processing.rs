use super::*;

const OCR_RESULT_IGNORED_EVENT: &str = "ocr_result_ignored_because_cancelled";
const OCR_RESULT_APPLIED_EVENT: &str = "ocr_result_applied";
const CHARACTER_POSITIONS_CALCULATED_EVENT: &str = "character_positions_calculated";
const OCR_FAILED_EVENT: &str = "ocr_failed";

impl InteractiveOcrView {
    pub fn set_ocr_result(&mut self, result: OcrResult) -> bool {
        if self.ocr_state != OcrState::Processing {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": OCR_RESULT_IGNORED_EVENT})
            );
            return false;
        }
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_RESULT_APPLIED_EVENT, "text_block_count": result.text_blocks.len()})
        );

        self.char_positions = Self::calculate_char_positions(&result);
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CHARACTER_POSITIONS_CALCULATED_EVENT, "character_position_count": self.char_positions.len()})
        );
        self.ocr_result = Some(result);
        self.ocr_state = OcrState::Completed;

        if !self.char_positions.is_empty() {
            self.show_help_hint = true;
        }
        true
    }

    pub fn set_ocr_failed(&mut self, error: String) -> bool {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_FAILED_EVENT, "error": error})
        );
        self.ocr_state = OcrState::Failed(error);
        true
    }

    fn calculate_char_positions(result: &OcrResult) -> Vec<CharPosition> {
        let mut positions = Vec::new();

        for (word_index, word) in result.text_blocks.iter().enumerate() {
            let char_count = word.content.chars().count();
            if char_count == 0 {
                continue;
            }

            let char_width = word.bounds.width / char_count as f32;

            for (char_index, character) in word.content.chars().enumerate() {
                let char_x = word.bounds.x + (char_index as f32 * char_width);
                positions.push(CharPosition {
                    word_index,
                    char_index,
                    bounds: Rectangle {
                        x: char_x,
                        y: word.bounds.y,
                        width: char_width,
                        height: word.bounds.height,
                    },
                    character,
                });
            }
        }

        positions
    }

    #[allow(dead_code)]
    fn detect_vertical_layout(&self, positions: &[&CharPosition]) -> bool {
        if positions.len() < 2 {
            return false;
        }

        let mut y_changes = 0;
        for index in 1..positions.len() {
            if (positions[index].bounds.y - positions[index - 1].bounds.y).abs() > 10.0 {
                y_changes += 1;
            }
        }

        y_changes as f32 / positions.len() as f32 > 0.3
    }
}
