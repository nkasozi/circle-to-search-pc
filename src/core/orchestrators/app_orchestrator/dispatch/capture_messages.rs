use super::*;

impl AppOrchestrator {
    pub(super) fn dispatch_capture_message(
        &mut self,
        message: OrchestratorMessage,
    ) -> DispatchOutcome {
        match message {
            OrchestratorMessage::CaptureScreen => Ok(self.handle_capture_screen()),
            OrchestratorMessage::PerformCapture => Ok(self.handle_perform_capture()),
            OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer) => {
                Ok(self.handle_open_capture_overlay(mouse_x, mouse_y, capture_buffer))
            }
            OrchestratorMessage::CaptureError(error_message) => {
                Ok(self.handle_capture_error(error_message))
            }
            OrchestratorMessage::CaptureOverlayMessage(window_id, capture_message) => {
                Ok(self.handle_capture_overlay_message(window_id, capture_message))
            }
            OrchestratorMessage::ConfirmSelection(overlay_id) => {
                Ok(self.handle_confirm_selection(overlay_id))
            }
            OrchestratorMessage::ShowCroppedImage(capture_buffer, selection_rectangle) => {
                Ok(self.handle_show_cropped_image(capture_buffer, selection_rectangle))
            }
            OrchestratorMessage::CopyImageToClipboard(window_id, buffer, draw_strokes) => {
                Ok(self.handle_copy_image_to_clipboard(window_id, buffer, draw_strokes))
            }
            OrchestratorMessage::SaveImageToFile(window_id, buffer, draw_strokes) => {
                Ok(self.handle_save_image_to_file(window_id, buffer, draw_strokes))
            }
            unhandled => Err(unhandled),
        }
    }
}
