use super::*;

impl InteractiveOcrView {
    pub fn set_ocr_result(&mut self, result: OcrResult) {
        log::info!(
            "[INTERACTIVE_OCR] Setting OCR result with {} text blocks",
            result.text_blocks.len()
        );

        self.char_positions = Self::calculate_char_positions(&result);
        log::info!(
            "[INTERACTIVE_OCR] Calculated {} character positions",
            self.char_positions.len()
        );
        self.ocr_result = Some(result);
        self.ocr_state = OcrState::Completed;

        if !self.char_positions.is_empty() {
            self.show_help_hint = true;
        }
    }

    pub fn set_ocr_failed(&mut self, error: String) {
        log::error!("[INTERACTIVE_OCR] OCR failed: {}", error);
        self.ocr_state = OcrState::Failed(error);
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
