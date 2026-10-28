use std::sync::Arc;

use iced::Task;

use super::{events, log_flow_event, RecordingFlow, RecordingMessage};
use crate::core::interfaces::ports::CaptureDevice;

impl RecordingFlow {
    pub(super) fn check_microphone_access(&self) -> Task<RecordingMessage> {
        let permission_gate = Arc::clone(&self.dependencies.permission_gate);
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    permission_gate.ensure_access(CaptureDevice::Microphone)
                })
                .await
                .map_err(|join_error| join_error.to_string())
                .and_then(|result| result.map_err(|error| error.to_string()))
            },
            RecordingMessage::MicrophoneAccessChecked,
        )
    }

    pub(super) fn handle_microphone_access_checked(
        &mut self,
        result: Result<bool, String>,
    ) -> Task<RecordingMessage> {
        let Err(reason) = result else {
            return Task::none();
        };
        log_flow_event(
            events::MICROPHONE_ACCESS_DENIED,
            serde_json::json!({"reason": reason}),
        );
        self.preferences.microphone_enabled = false;
        let toggles = self.setup_toggles();
        if let Some(setup_view) = self.setup_view.as_mut() {
            setup_view.toggles = toggles;
            setup_view.notice = Some(reason);
        }
        Task::none()
    }
}
