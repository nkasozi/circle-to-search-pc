use super::*;

mod capture_messages;
mod media_messages;
mod ocr_messages;
mod settings_messages;
mod task_free_messages;
mod window_messages;
mod window_picker_messages;

type DispatchOutcome = Result<Task<OrchestratorMessage>, OrchestratorMessage>;

const EVENT_ORCHESTRATOR_MESSAGE_RECEIVED: &str = "orchestrator_message_received";

impl AppOrchestrator {
    pub fn update(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        let _ = self.refresh_correlation_id();
        if is_high_frequency_message(&message) {
            return self.update_quietly(message);
        }
        let _ = self.log_info_event(
            EVENT_ORCHESTRATOR_MESSAGE_RECEIVED,
            serde_json::json!({
                "message": format!("{:?}", message),
            }),
        );
        self.dispatch_message(message)
    }

    fn dispatch_message(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        self.dispatch_window_message(message)
            .or_else(|message| self.dispatch_capture_message(message))
            .or_else(|message| self.dispatch_ocr_message(message))
            .or_else(|message| self.dispatch_window_picker_message(message))
            .or_else(|message| self.dispatch_media_message(message))
            .or_else(|message| self.dispatch_settings_message(message))
            .unwrap_or_else(|message| self.dispatch_task_free_message(message))
    }

    fn update_quietly(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        match message {
            OrchestratorMessage::Recording(recording_message) => {
                self.handle_recording_message(recording_message)
            }
            OrchestratorMessage::SpinnerTick => self.handle_spinner_tick(),
            OrchestratorMessage::PlayerTick => self.handle_player_tick(),
            OrchestratorMessage::PlayerFrameAllocated(window_id, allocation) => {
                self.handle_player_frame_allocated(window_id, allocation)
            }
            OrchestratorMessage::KeyPressedInWindow(window_id, modifiers, physical_code) => {
                self.handle_key_pressed_in_window(window_id, modifiers, physical_code)
            }
            _ => Task::none(),
        }
    }

    fn handle_spinner_tick(&mut self) -> Task<OrchestratorMessage> {
        for window in self.windows.values_mut() {
            let _ = advance_window_spinner(window);
        }
        Task::none()
    }
}

fn advance_window_spinner(window: &mut AppWindow) -> bool {
    match window {
        AppWindow::InteractiveOcr(view) => {
            view.update(crate::presentation::InteractiveOcrMessage::SpinnerTick);
            true
        }
        AppWindow::WindowPicker(view) => {
            view.update(crate::presentation::WindowPickerMessage::SpinnerTick);
            true
        }
        _ => false,
    }
}

fn is_high_frequency_message(message: &OrchestratorMessage) -> bool {
    matches!(
        message,
        OrchestratorMessage::SpinnerTick
            | OrchestratorMessage::Recording(RecordingMessage::FrameTick)
            | OrchestratorMessage::KeyPressedInWindow(..)
            | OrchestratorMessage::PlayerTick
            | OrchestratorMessage::PlayerFrameAllocated(..)
    )
}
