use iced::{window, Task};

use super::session::MISSING_PLAN_ERROR;
use super::windows::close_if_open;
use super::{events, log_flow_event, RecordingFlow, RecordingMessage, RecordingPhase};
use crate::core::models::FinishedRecording;
use crate::presentation::recording::controls_view::ControlsPhase;

impl RecordingFlow {
    pub(super) fn request_stop(&mut self) -> Task<RecordingMessage> {
        let RecordingPhase::Recording { session } = self.phase.clone() else {
            return Task::none();
        };
        self.phase = RecordingPhase::Finishing;
        self.controls_view.phase = ControlsPhase::Saving;
        self.annotation_view.drawing_enabled = false;
        log_flow_event(events::RECORDING_STOP_REQUESTED, serde_json::json!({}));
        let passthrough = close_drawing_passthrough(self.windows.annotation);
        let stop_task = Task::perform(
            async move {
                tokio::task::spawn_blocking(move || match session.take() {
                    Some(active_session) => active_session
                        .stop()
                        .map_err(|error| format!("{:#}", error)),
                    None => Err(MISSING_PLAN_ERROR.to_string()),
                })
                .await
                .map_err(|join_error| join_error.to_string())
                .and_then(|result| result)
            },
            RecordingMessage::SessionFinished,
        );
        Task::batch([passthrough, stop_task])
    }

    pub(super) fn request_discard(&mut self) -> Task<RecordingMessage> {
        let RecordingPhase::Recording { session } = self.phase.clone() else {
            return Task::none();
        };
        self.phase = RecordingPhase::Finishing;
        self.controls_view.phase = ControlsPhase::Saving;
        log_flow_event(events::RECORDING_DISCARD_REQUESTED, serde_json::json!({}));
        Task::perform(
            async move {
                let _ = tokio::task::spawn_blocking(move || {
                    session
                        .take()
                        .map(|active_session| active_session.discard())
                })
                .await;
            },
            |_| RecordingMessage::SessionDiscarded,
        )
    }

    pub(super) fn handle_session_finished(
        &mut self,
        result: Result<FinishedRecording, String>,
    ) -> Task<RecordingMessage> {
        let cleanup = self.close_recording_windows();
        let follow_up = match result {
            Ok(finished) => {
                self.has_unreported_save = true;
                self.show_saved_card(finished)
            }
            Err(reason) => self.show_failure_card(reason),
        };
        Task::batch([cleanup, follow_up])
    }

    pub(super) fn handle_session_discarded(&mut self) -> Task<RecordingMessage> {
        log_flow_event(events::RECORDING_DISCARDED, serde_json::json!({}));
        self.close_recording_windows()
    }

    pub(super) fn close_recording_windows(&mut self) -> Task<RecordingMessage> {
        self.phase = RecordingPhase::Idle;
        let close_controls = close_if_open(self.windows.controls.take());
        let close_annotation = close_if_open(self.windows.annotation.take());
        let stop_camera = self.stop_camera();
        Task::batch([close_controls, close_annotation, stop_camera])
    }

    pub(super) fn update_session_status(&mut self) -> Task<RecordingMessage> {
        let RecordingPhase::Recording { session } = &self.phase else {
            return Task::none();
        };
        let status = session.with_session(|active_session| {
            (
                active_session.elapsed(),
                active_session.is_paused(),
                active_session.failure(),
            )
        });
        let Some((elapsed, is_paused, failure)) = status else {
            return Task::none();
        };
        self.controls_view.phase = ControlsPhase::Recording { elapsed, is_paused };
        match failure {
            Some(reason) => {
                log_flow_event(
                    events::RECORDING_FAILED_MIDWAY,
                    serde_json::json!({"error": reason}),
                );
                self.request_stop()
            }
            None => Task::none(),
        }
    }
}

fn close_drawing_passthrough(annotation_window: Option<window::Id>) -> Task<RecordingMessage> {
    annotation_window
        .map(window::enable_mouse_passthrough)
        .unwrap_or_else(Task::none)
}
