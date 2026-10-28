use super::*;

impl AppOrchestrator {
    pub(super) fn dispatch_ocr_message(&mut self, message: OrchestratorMessage) -> DispatchOutcome {
        match message {
            OrchestratorMessage::ProcessOcr(window_id, buffer) => {
                Ok(self.handle_process_ocr(window_id, buffer))
            }
            OrchestratorMessage::OcrComplete(window_id, result) => {
                Ok(self.handle_ocr_complete(window_id, result))
            }
            OrchestratorMessage::OcrServiceReady(service) => {
                Ok(self.handle_ocr_service_ready(service))
            }
            OrchestratorMessage::OcrServiceFailed(error) => {
                Ok(self.handle_ocr_service_failed(error))
            }
            OrchestratorMessage::InteractiveOcrMessage(window_id, ocr_message) => {
                Ok(self.handle_interactive_ocr_message(window_id, ocr_message))
            }
            OrchestratorMessage::PerformImageSearch(window_id, buffer, query) => {
                Ok(self.handle_perform_image_search(window_id, buffer, query))
            }
            OrchestratorMessage::SpinnerTick => Ok(self.handle_spinner_tick()),
            unhandled => Err(unhandled),
        }
    }
}
