use super::*;

mod drawing;
mod export;
mod ocr;
mod search;
mod selection;

impl InteractiveOcrView {
    pub fn update(&mut self, message: InteractiveOcrMessage) -> bool {
        self.try_apply_selection_message(message)
            .and_then(|remaining_message| self.try_apply_drawing_message(remaining_message))
            .and_then(|remaining_message| self.try_apply_search_message(remaining_message))
            .and_then(|remaining_message| self.try_apply_export_message(remaining_message))
            .and_then(|remaining_message| self.try_apply_ocr_message(remaining_message))
            .is_none()
    }
}
