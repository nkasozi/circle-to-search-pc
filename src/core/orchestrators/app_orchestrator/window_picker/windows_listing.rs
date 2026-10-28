use super::super::*;

#[derive(Clone, Copy)]
pub(super) struct WindowsListEvents {
    pub(super) success_event: &'static str,
    pub(super) failure_event: &'static str,
}

pub(super) fn build_windows_list_message(
    screen_capturer: &dyn ScreenCapturer,
    picker_window_id: Id,
    correlation_id: &str,
    events: WindowsListEvents,
) -> OrchestratorMessage {
    match screen_capturer.list_capturable_windows() {
        Ok(windows) => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id.to_string(),
                events.success_event,
                serde_json::json!({
                    "window_count": windows.len(),
                }),
            );
            OrchestratorMessage::WindowsListLoaded(picker_window_id, windows)
        }
        Err(error) => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id.to_string(),
                events.failure_event,
                serde_json::json!({
                    "error": error.to_string(),
                }),
            );
            OrchestratorMessage::WindowsListLoaded(picker_window_id, vec![])
        }
    }
}
