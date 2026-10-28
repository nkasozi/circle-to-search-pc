use super::*;

const EVENT_ORCHESTRATOR_MESSAGE_COMPLETED: &str = "orchestrator_message_completed";
const EVENT_ORCHESTRATOR_MESSAGE_UNROUTED: &str = "orchestrator_message_unrouted";
const EVENT_REAPPLY_HOTKEYS_HANDLED_BY_APP: &str = "reapply_hotkeys_handled_by_app";
const EVENT_WINDOW_CAPTURE_FAILED: &str = "window_capture_failed";
const COMPLETION_RESULT_NO_TASK: &str = "no_task";

impl AppOrchestrator {
    pub(super) fn dispatch_task_free_message(
        &mut self,
        message: OrchestratorMessage,
    ) -> Task<OrchestratorMessage> {
        let was_handled = self.apply_task_free_message(message);
        if !was_handled {
            return Task::none();
        }
        let _ = self.log_info_event(
            EVENT_ORCHESTRATOR_MESSAGE_COMPLETED,
            serde_json::json!({
                "result": COMPLETION_RESULT_NO_TASK,
            }),
        );
        Task::none()
    }

    fn apply_task_free_message(&mut self, message: OrchestratorMessage) -> bool {
        match message {
            OrchestratorMessage::RecentRecordingsLoaded(recordings) => {
                self.recent_recordings = recordings;
                true
            }
            OrchestratorMessage::ReapplyHotkeys => {
                self.log_debug_event(EVENT_REAPPLY_HOTKEYS_HANDLED_BY_APP, serde_json::json!({}))
            }
            OrchestratorMessage::WindowCaptureError(error_message) => self.log_error_event(
                EVENT_WINDOW_CAPTURE_FAILED,
                serde_json::json!({
                    "error": error_message,
                }),
            ),
            unrouted => {
                let _ = self.log_error_event(
                    EVENT_ORCHESTRATOR_MESSAGE_UNROUTED,
                    serde_json::json!({
                        "message": format!("{:?}", unrouted),
                    }),
                );
                false
            }
        }
    }
}
