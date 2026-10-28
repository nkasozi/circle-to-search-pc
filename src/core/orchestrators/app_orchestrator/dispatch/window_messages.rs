use super::*;

const LOG_TAG: &str = "[ORCHESTRATOR]";
const WINDOW_CLOSING_EVENT: &str = "window_closing";

impl AppOrchestrator {
    pub(super) fn dispatch_window_message(
        &mut self,
        message: OrchestratorMessage,
    ) -> DispatchOutcome {
        match message {
            OrchestratorMessage::OpenMainWindow => Ok(self.handle_open_main_window()),
            OrchestratorMessage::CreateHiddenWindow => Ok(self.create_hidden_window()),
            OrchestratorMessage::CloseWindow(window_id) => {
                log::info!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": WINDOW_CLOSING_EVENT, "window_id": format!("{:?}", window_id), "correlation_id": self.current_correlation_id})
                );
                Ok(window::close(window_id))
            }
            OrchestratorMessage::WindowClosed(window_id) => {
                Ok(self.handle_window_closed(window_id))
            }
            OrchestratorMessage::WindowFocused(window_id) => {
                Ok(self.handle_window_focused(window_id))
            }
            OrchestratorMessage::TrayEvent(event) => Ok(self.handle_tray_event(event)),
            OrchestratorMessage::HideMainWindow => Ok(self.handle_hide_main_window()),
            OrchestratorMessage::OpenOnboarding => Ok(self.handle_open_onboarding()),
            OrchestratorMessage::OnboardingMsg(window_id, onboarding_message) => {
                Ok(self.handle_onboarding_message(window_id, onboarding_message))
            }
            OrchestratorMessage::EscapePressed(window_id) => {
                Ok(self.handle_escape_pressed(window_id))
            }
            unhandled => Err(unhandled),
        }
    }
}
