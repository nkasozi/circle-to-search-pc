use super::*;

impl AppOrchestrator {
    pub(super) fn dispatch_media_message(
        &mut self,
        message: OrchestratorMessage,
    ) -> DispatchOutcome {
        match message {
            OrchestratorMessage::Recording(recording_message) => {
                Ok(self.handle_recording_message(recording_message))
            }
            OrchestratorMessage::Home(home_message) => Ok(self.handle_home_message(home_message)),
            OrchestratorMessage::Player(window_id, player_message) => {
                Ok(self.handle_player_message(window_id, player_message))
            }
            OrchestratorMessage::PlayerTick => Ok(self.handle_player_tick()),
            OrchestratorMessage::PlayerFrameAllocated(window_id, allocation) => {
                Ok(self.handle_player_frame_allocated(window_id, allocation))
            }
            unhandled => Err(unhandled),
        }
    }
}
