use super::*;
use crate::core::interfaces::ports::RecentRecordingsQuery;

const RECENT_RECORDINGS_LIMIT: usize = 6;
const EVENT_WINDOWS_MINIMIZED_FOR_RECORDING: &str = "windows_minimized_for_recording";
const EVENT_RECORDING_PREFERENCES_SAVE_FAILED: &str = "recording_preferences_save_failed";

impl AppOrchestrator {
    pub(super) fn handle_recording_message(
        &mut self,
        message: RecordingMessage,
    ) -> Task<OrchestratorMessage> {
        let minimize_task = self.minimize_windows_when_starting_recorder(&message);
        let flow_task = self
            .recording_flow
            .update(message)
            .map(OrchestratorMessage::Recording);
        let persist_task = self.persist_recording_preferences();
        let refresh_task = self.refresh_recordings_after_save();
        let player_task = self.open_requested_playback();
        Task::batch([
            minimize_task,
            flow_task,
            persist_task,
            refresh_task,
            player_task,
        ])
    }

    fn minimize_windows_when_starting_recorder(
        &self,
        message: &RecordingMessage,
    ) -> Task<OrchestratorMessage> {
        let is_starting_recorder =
            matches!(message, RecordingMessage::Toggle) && self.recording_flow.is_idle();
        match is_starting_recorder {
            true => self.minimize_windows_for_recording(),
            false => Task::none(),
        }
    }

    fn refresh_recordings_after_save(&mut self) -> Task<OrchestratorMessage> {
        match self.recording_flow.take_saved_notification() {
            true => self.refresh_recent_recordings(),
            false => Task::none(),
        }
    }

    fn open_requested_playback(&mut self) -> Task<OrchestratorMessage> {
        match self.recording_flow.take_playback_request() {
            Some(path) => self.open_player(path),
            None => Task::none(),
        }
    }

    fn minimize_windows_for_recording(&self) -> Task<OrchestratorMessage> {
        let window_ids: Vec<Id> = self
            .windows
            .iter()
            .filter(|(_, window)| is_minimizable_for_recording(window))
            .map(|(window_id, _)| *window_id)
            .collect();
        let _ = self.log_info_event(
            EVENT_WINDOWS_MINIMIZED_FOR_RECORDING,
            serde_json::json!({"count": window_ids.len()}),
        );
        Task::batch(
            window_ids
                .into_iter()
                .map(|window_id| window::minimize(window_id, true)),
        )
    }

    fn persist_recording_preferences(&mut self) -> Task<OrchestratorMessage> {
        let flow_preferences = self.recording_flow.preferences();
        if flow_preferences == self.settings.recording_preferences {
            return Task::none();
        }
        self.settings.recording_preferences = flow_preferences;
        if let Err(save_error) = self.settings.save() {
            let _ = self.log_error_event(
                EVENT_RECORDING_PREFERENCES_SAVE_FAILED,
                serde_json::json!({"error": save_error.to_string()}),
            );
        }
        Task::none()
    }

    pub(super) fn refresh_recent_recordings(&self) -> Task<OrchestratorMessage> {
        let library = Arc::clone(&self.recording_library);
        let query = RecentRecordingsQuery {
            recordings_directory: std::path::PathBuf::from(&self.settings.recording_save_location),
            maximum_results: RECENT_RECORDINGS_LIMIT,
        };
        Task::perform(
            async move { library.list_recent_recordings(&query).unwrap_or_default() },
            OrchestratorMessage::RecentRecordingsLoaded,
        )
    }

    pub(super) fn handle_toggle_recording_hotkey(&mut self) -> Task<OrchestratorMessage> {
        self.handle_recording_message(RecordingMessage::Toggle)
    }

    pub fn recording_subscription(&self) -> iced::Subscription<OrchestratorMessage> {
        self.recording_flow
            .subscription()
            .map(OrchestratorMessage::Recording)
    }

    pub fn is_recording(&self) -> bool {
        self.recording_flow.is_recording()
    }
}

fn is_minimizable_for_recording(window: &AppWindow) -> bool {
    matches!(
        window,
        AppWindow::Main
            | AppWindow::Settings(_)
            | AppWindow::InteractiveOcr(_)
            | AppWindow::WindowPicker(_)
            | AppWindow::Player(_)
    )
}
