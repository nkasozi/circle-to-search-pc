use super::*;

const OCR_START_REQUESTED_EVENT: &str = "ocr_start_requested";
const OCR_CANCEL_REQUESTED_EVENT: &str = "ocr_cancel_requested";
const OCR_OVERLAY_CLEARED_EVENT: &str = "ocr_overlay_cleared";
const OCR_FAILED_EVENT: &str = "ocr_failed";
const OCR_RETRY_REQUESTED_EVENT: &str = "ocr_retry_requested";
impl InteractiveOcrView {
    pub(super) fn try_apply_ocr_message(
        &mut self,
        message: InteractiveOcrMessage,
    ) -> Option<InteractiveOcrMessage> {
        let _ocr_changed = match message {
            InteractiveOcrMessage::StartOcr => self.handle_start_ocr(),
            InteractiveOcrMessage::CancelOcr => self.handle_cancel_ocr(),
            InteractiveOcrMessage::ClearOcrOverlay => self.handle_clear_ocr_overlay(),
            InteractiveOcrMessage::OcrFailed(error) => self.handle_ocr_failed(error),
            InteractiveOcrMessage::RetryOcr => self.handle_retry_ocr(),
            other_message => return Some(other_message),
        };
        None
    }

    fn handle_start_ocr(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_START_REQUESTED_EVENT})
        );
        self.ocr_state = OcrState::Processing;
        true
    }

    fn handle_cancel_ocr(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_CANCEL_REQUESTED_EVENT})
        );
        self.ocr_state = OcrState::Idle;
        true
    }

    fn handle_clear_ocr_overlay(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_OVERLAY_CLEARED_EVENT})
        );
        self.ocr_result = None;
        self.char_positions.clear();
        self.selected_chars.clear();
        self.ocr_state = OcrState::Idle;
        true
    }

    fn handle_ocr_failed(&mut self, error: String) -> bool {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_FAILED_EVENT, "error": error})
        );
        self.ocr_state = OcrState::Failed(error);
        true
    }

    fn handle_retry_ocr(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_RETRY_REQUESTED_EVENT})
        );
        self.ocr_state = OcrState::Idle;
        self.ocr_result = None;
        self.char_positions.clear();
        self.selected_chars.clear();
        true
    }
}
