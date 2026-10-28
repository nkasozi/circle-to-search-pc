use super::*;

impl AppOrchestrator {
    pub(super) fn dispatch_settings_message(
        &mut self,
        message: OrchestratorMessage,
    ) -> DispatchOutcome {
        match message {
            OrchestratorMessage::GlobalHotkey(action) => Ok(self.handle_global_hotkey(action)),
            OrchestratorMessage::HotkeysApplied(failures) => {
                Ok(self.handle_hotkeys_applied(failures))
            }
            OrchestratorMessage::Settings(settings_message) => {
                Ok(self.handle_settings_message(settings_message))
            }
            OrchestratorMessage::KeyPressedInWindow(window_id, modifiers, physical_code) => {
                Ok(self.handle_key_pressed_in_window(window_id, modifiers, physical_code))
            }
            unhandled => Err(unhandled),
        }
    }
}
