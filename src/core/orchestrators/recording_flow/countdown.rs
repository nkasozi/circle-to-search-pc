use iced::Task;

use super::windows::close_if_open;
use super::{events, log_flow_event, RecordingFlow, RecordingMessage, RecordingPhase};
use crate::core::models::CaptureMode;

impl RecordingFlow {
    pub(super) fn begin_countdown(&mut self) -> Task<RecordingMessage> {
        let Some(capture_rectangle) = self
            .setup_view
            .as_ref()
            .and_then(|view| view.capture_rectangle())
        else {
            return Task::none();
        };
        if let Some(setup_view) = self.setup_view.as_ref() {
            self.preferences.capture_mode = setup_view.mode;
        }
        self.capture_rectangle = Some(capture_rectangle);
        let countdown_seconds = self.preferences.countdown.seconds();
        log_flow_event(
            events::RECORDING_COUNTDOWN_STARTED,
            serde_json::json!({"seconds": countdown_seconds}),
        );
        if countdown_seconds == 0 {
            return self.finish_countdown();
        }
        self.phase = RecordingPhase::Countdown {
            remaining: countdown_seconds,
        };
        if let Some(setup_view) = self.setup_view.as_mut() {
            setup_view.countdown_remaining = Some(countdown_seconds);
        }
        Task::none()
    }

    pub(super) fn handle_countdown_tick(&mut self) -> Task<RecordingMessage> {
        let RecordingPhase::Countdown { remaining } = self.phase else {
            return Task::none();
        };
        let next_remaining = remaining.saturating_sub(1);
        if next_remaining == 0 {
            return self.finish_countdown();
        }
        self.phase = RecordingPhase::Countdown {
            remaining: next_remaining,
        };
        if let Some(setup_view) = self.setup_view.as_mut() {
            setup_view.countdown_remaining = Some(next_remaining);
        }
        Task::none()
    }

    pub(super) fn finish_countdown(&mut self) -> Task<RecordingMessage> {
        if self.capture_rectangle.is_none() {
            return Task::none();
        }
        let close_setup = close_if_open(self.windows.setup.take());
        self.setup_view = None;
        self.phase = RecordingPhase::Starting;
        Task::batch([close_setup, self.open_recording_windows()])
    }

    pub(super) fn selected_capture_rectangle(&self) -> Option<iced::Rectangle> {
        let setup_view = self.setup_view.as_ref()?;
        match setup_view.mode {
            CaptureMode::FullScreen => setup_view.capture_rectangle(),
            CaptureMode::Region => setup_view.selected_region,
        }
    }
}
