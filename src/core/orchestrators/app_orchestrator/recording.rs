use iced::window::{self, Id};
use iced::{Size, Task};

use super::{AppOrchestrator, AppWindow, OrchestratorMessage};
use crate::presentation::{RecordingView, RecordingViewMessage};

const RECORDING_WINDOW_WIDTH: f32 = 900.0;
const RECORDING_WINDOW_HEIGHT: f32 = 650.0;

impl AppOrchestrator {
    pub(super) fn handle_open_recording(&mut self) -> Task<OrchestratorMessage> {
        if self.recording_window_id.is_some() {
            self.log_info_event("recording_window_already_open", serde_json::json!({}));
            return Task::none();
        }

        self.log_info_event("opening_recording_window", serde_json::json!({}));

        let recording_view = RecordingView::build(true, false);

        let (window_id, open_task) = window::open(window::Settings {
            size: Size::new(RECORDING_WINDOW_WIDTH, RECORDING_WINDOW_HEIGHT),
            resizable: true,
            decorations: true,
            ..Default::default()
        });

        self.recording_window_id = Some(window_id);
        self.windows
            .insert(window_id, AppWindow::Recording(recording_view));

        open_task.discard()
    }

    pub(super) fn handle_recording_view_message(
        &mut self,
        window_id: Id,
        message: RecordingViewMessage,
    ) -> Task<OrchestratorMessage> {
        let should_close = matches!(message, RecordingViewMessage::Close);

        if let Some(AppWindow::Recording(recording_view)) = self.windows.get_mut(&window_id) {
            recording_view.update(message);
        }

        if should_close {
            return self.close_recording_window(window_id);
        }

        Task::none()
    }

    pub(super) fn handle_recording_completed(
        &mut self,
        window_id: Id,
        output_path: String,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "recording_session_completed",
            serde_json::json!({
                "output_path": output_path,
            }),
        );

        if let Some(AppWindow::Recording(recording_view)) = self.windows.get_mut(&window_id) {
            recording_view.update(RecordingViewMessage::RecordingCompleted(output_path));
        }

        Task::none()
    }

    pub(super) fn handle_recording_failed(
        &mut self,
        window_id: Id,
        error: String,
    ) -> Task<OrchestratorMessage> {
        self.log_error_event(
            "recording_session_failed",
            serde_json::json!({
                "error": error,
            }),
        );

        if let Some(AppWindow::Recording(recording_view)) = self.windows.get_mut(&window_id) {
            recording_view.update(RecordingViewMessage::RecordingFailed(error));
        }

        Task::none()
    }

    fn close_recording_window(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event("closing_recording_window", serde_json::json!({}));

        self.windows.remove(&window_id);

        if self.recording_window_id == Some(window_id) {
            self.recording_window_id = None;
        }

        window::close(window_id)
    }
}
