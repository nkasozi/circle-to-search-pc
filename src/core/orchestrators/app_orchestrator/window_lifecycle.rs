use super::*;
use crate::adapters::macos_app_behavior;

mod app_control;
mod main_window;
mod window_closing;

const EVENT_CAPTURE_RESULT_WINDOWS_RECOVERING: &str =
    "app_window_focused_recovering_capture_result_windows";

impl AppOrchestrator {
    pub(super) fn handle_window_focused(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let is_hidden_window = self.hidden_window_id == Some(window_id);
        let is_main_window = self.main_window_id == Some(window_id);

        if !is_hidden_window && !is_main_window {
            return Task::none();
        }

        let ocr_window_ids = self.collect_capture_result_window_ids();

        if ocr_window_ids.is_empty() {
            return Task::none();
        }

        let _ = self.log_info_event(
            EVENT_CAPTURE_RESULT_WINDOWS_RECOVERING,
            serde_json::json!({"ocr_window_count": ocr_window_ids.len()}),
        );

        let focus_tasks: Vec<Task<OrchestratorMessage>> = ocr_window_ids
            .iter()
            .map(|ocr_window_id| restore_and_focus_window(*ocr_window_id))
            .collect();

        macos_app_behavior::macos::activate_app_ignoring_other_apps();

        Task::batch(focus_tasks)
    }

    fn collect_capture_result_window_ids(&self) -> Vec<Id> {
        self.windows
            .iter()
            .filter_map(|(window_id, window)| {
                matches!(window, AppWindow::InteractiveOcr(_)).then_some(*window_id)
            })
            .collect()
    }
}

fn restore_and_focus_window(window_id: Id) -> Task<OrchestratorMessage> {
    window::minimize(window_id, false).chain(window::gain_focus(window_id))
}
