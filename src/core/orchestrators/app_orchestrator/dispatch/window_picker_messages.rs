use super::*;

impl AppOrchestrator {
    pub(super) fn dispatch_window_picker_message(
        &mut self,
        message: OrchestratorMessage,
    ) -> DispatchOutcome {
        match message {
            OrchestratorMessage::OpenWindowPicker => Ok(self.handle_open_window_picker()),
            OrchestratorMessage::WindowPickerMsg(window_id, picker_message) => {
                Ok(self.handle_window_picker_message(window_id, picker_message))
            }
            OrchestratorMessage::WindowsListLoaded(window_id, windows) => {
                Ok(self.handle_windows_list_loaded(window_id, windows))
            }
            OrchestratorMessage::FocusWindowAndCapture(app_name) => {
                Ok(self.handle_focus_window_and_capture(app_name))
            }
            OrchestratorMessage::CaptureSelectedWindow(window_id) => {
                Ok(self.handle_capture_selected_window(window_id))
            }
            OrchestratorMessage::WindowCaptureComplete(capture_buffer) => {
                Ok(self.handle_window_capture_complete(capture_buffer))
            }
            unhandled => Err(unhandled),
        }
    }
}
