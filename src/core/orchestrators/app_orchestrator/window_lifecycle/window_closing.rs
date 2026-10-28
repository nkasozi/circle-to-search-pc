use super::*;

const EVENT_WINDOW_CLOSED: &str = "window_closed";
const EVENT_HIDDEN_WINDOW_CLOSED_UNEXPECTEDLY: &str = "hidden_window_closed_unexpectedly";
const EVENT_MAIN_WINDOW_CLOSED: &str = "main_window_closed";
const EVENT_ONBOARDING_WINDOW_CLOSED: &str = "onboarding_window_closed";
const EVENT_WINDOW_PICKER_CLOSED: &str = "window_picker_closed";
const EVENT_WINDOW_REMOVED_FROM_TRACKING: &str = "window_removed_from_tracking";

#[derive(Clone, Copy)]
enum SingletonWindow {
    Main,
    Onboarding,
    WindowPicker,
}

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_window_closed(
        &mut self,
        window_id: Id,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(
            EVENT_WINDOW_CLOSED,
            serde_json::json!({"window_id": format!("{:?}", window_id)}),
        );
        if self.recording_flow.owns_window(window_id) {
            return self.handle_recording_message(RecordingMessage::WindowClosed(window_id));
        }

        if Some(window_id) == self.hidden_window_id {
            return self.recover_closed_hidden_window(window_id);
        }

        if self.forget_closed_singleton_window(window_id) {
            return Task::none();
        }

        self.forget_closed_tracked_window(window_id)
    }

    fn recover_closed_hidden_window(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let _ = self.log_error_event(
            EVENT_HIDDEN_WINDOW_CLOSED_UNEXPECTEDLY,
            serde_json::json!({"window_id": format!("{:?}", window_id)}),
        );
        self.hidden_window_id = None;
        self.windows.remove(&window_id);
        self.create_hidden_window()
    }

    fn forget_closed_singleton_window(&mut self, window_id: Id) -> bool {
        let Some(singleton_window) = self.identify_singleton_window(window_id) else {
            return false;
        };
        let closed_event = self.clear_singleton_window_id(singleton_window);
        let _ = self.log_info_event(closed_event, serde_json::json!({}));
        self.windows.remove(&window_id);
        true
    }

    fn identify_singleton_window(&self, window_id: Id) -> Option<SingletonWindow> {
        match Some(window_id) {
            closed_id if closed_id == self.main_window_id => Some(SingletonWindow::Main),
            closed_id if closed_id == self.onboarding_window_id => {
                Some(SingletonWindow::Onboarding)
            }
            closed_id if closed_id == self.window_picker_window_id => {
                Some(SingletonWindow::WindowPicker)
            }
            _ => None,
        }
    }

    fn clear_singleton_window_id(&mut self, singleton_window: SingletonWindow) -> &'static str {
        match singleton_window {
            SingletonWindow::Main => {
                self.main_window_id = None;
                EVENT_MAIN_WINDOW_CLOSED
            }
            SingletonWindow::Onboarding => {
                self.onboarding_window_id = None;
                EVENT_ONBOARDING_WINDOW_CLOSED
            }
            SingletonWindow::WindowPicker => {
                self.window_picker_window_id = None;
                EVENT_WINDOW_PICKER_CLOSED
            }
        }
    }

    fn forget_closed_tracked_window(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let was_ocr_window = matches!(
            self.windows.get(&window_id),
            Some(AppWindow::InteractiveOcr(_))
        );
        self.windows.remove(&window_id);
        if Some(window_id) == self.settings_window_id {
            self.settings_window_id = None;
            let _ = self.discard_settings_edit();
        }
        let _ = self.log_info_event(
            EVENT_WINDOW_REMOVED_FROM_TRACKING,
            serde_json::json!({"remaining_windows": self.windows.len()}),
        );
        self.status = global_constants::STATUS_READY.to_string();

        if !was_ocr_window {
            return Task::none();
        }

        let Some(main_window_id) = self.main_window_id else {
            return Task::none();
        };

        window::minimize(main_window_id, false)
    }
}
