use std::time::Duration;

use iced::{Subscription, Task};

use super::{RecordingFlow, RecordingMessage, RecordingPhase};

const FRAME_TICK_INTERVAL: Duration = Duration::from_millis(33);
const COUNTDOWN_TICK_INTERVAL: Duration = Duration::from_secs(1);

impl RecordingFlow {
    pub fn update(&mut self, message: RecordingMessage) -> Task<RecordingMessage> {
        match message {
            RecordingMessage::Toggle => self.handle_toggle(),
            RecordingMessage::Setup(setup_message) => self.handle_setup_message(setup_message),
            RecordingMessage::CountdownTick => self.handle_countdown_tick(),
            RecordingMessage::CameraBubble(bubble_message) => {
                self.handle_camera_bubble_message(bubble_message)
            }
            RecordingMessage::CameraStarted(result) => self.handle_camera_started(result),
            RecordingMessage::MicrophoneAccessChecked(result) => {
                self.handle_microphone_access_checked(result)
            }
            RecordingMessage::FrameTick => self.handle_frame_tick(),
            RecordingMessage::BeginSession => self.handle_begin_session(),
            RecordingMessage::SessionStarted(result) => self.handle_session_started(result),
            RecordingMessage::Controls(controls_message) => {
                self.handle_controls_message(controls_message)
            }
            RecordingMessage::Annotation(annotation_message) => {
                self.handle_annotation_message(annotation_message)
            }
            RecordingMessage::SessionFinished(result) => self.handle_session_finished(result),
            RecordingMessage::SessionDiscarded => self.handle_session_discarded(),
            RecordingMessage::ThumbnailReady(path, thumbnail) => {
                self.handle_thumbnail_ready(path, thumbnail)
            }
            RecordingMessage::Ready(ready_message) => self.handle_ready_message(ready_message),
            RecordingMessage::RecordingDeleted(result) => self.handle_recording_deleted(result),
            RecordingMessage::EscapePressed(window_id) => self.handle_escape(window_id),
            RecordingMessage::WindowClosed(window_id) => self.handle_window_closed(window_id),
        }
    }

    fn frame_tick_subscription(&self) -> Subscription<RecordingMessage> {
        let needs_frame_ticks = self.camera_capture.is_some()
            || matches!(
                self.phase,
                RecordingPhase::Recording { .. } | RecordingPhase::Starting
            );
        match needs_frame_ticks {
            true => iced::time::every(FRAME_TICK_INTERVAL).map(|_| RecordingMessage::FrameTick),
            false => Subscription::none(),
        }
    }

    fn countdown_tick_subscription(&self) -> Subscription<RecordingMessage> {
        match self.phase {
            RecordingPhase::Countdown { .. } => {
                iced::time::every(COUNTDOWN_TICK_INTERVAL).map(|_| RecordingMessage::CountdownTick)
            }
            _ => Subscription::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<RecordingMessage> {
        Subscription::batch([
            self.frame_tick_subscription(),
            self.countdown_tick_subscription(),
        ])
    }
}
