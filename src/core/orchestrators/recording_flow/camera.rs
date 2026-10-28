use std::sync::Arc;

use iced::widget::image;
use iced::{window, Size, Task};

use super::handles::CaptureHandle;
use super::windows::{close_if_open, floating_window_settings, open_window};
use super::{events, log_flow_event, RecordingFlow, RecordingMessage};
use crate::core::interfaces::ports::MediaSink;
use crate::presentation::recording::camera_bubble_view::{
    CameraBubbleContent, CameraBubbleMessage,
};

impl RecordingFlow {
    pub(super) fn toggle_camera(&mut self) -> Task<RecordingMessage> {
        self.preferences.camera_enabled = !self.preferences.camera_enabled;
        let camera_enabled = self.preferences.camera_enabled;
        if let Some(setup_view) = self.setup_view.as_mut() {
            setup_view.toggles.camera = camera_enabled;
        }
        match camera_enabled {
            true => self.start_camera(),
            false => self.stop_camera(),
        }
    }

    pub(super) fn start_camera(&mut self) -> Task<RecordingMessage> {
        let Some(display) = self.display else {
            return Task::none();
        };
        if self.windows.camera.is_some() {
            return Task::none();
        }
        let edge = self.preferences.camera_bubble_size.edge_in_points();
        let anchor = self
            .selected_capture_rectangle()
            .unwrap_or_else(|| display.full_rectangle());
        let (camera_window, open_task) = open_window(floating_window_settings(
            display.camera_bubble_position(anchor, edge),
            Size::new(edge, edge),
        ));
        self.windows.camera = Some(camera_window);
        self.camera_view.edge = edge;
        self.camera_view.shape = self.preferences.camera_bubble_shape;
        self.camera_view.content = CameraBubbleContent::Starting;
        self.camera_user_positioned = false;
        self.camera_preview =
            Arc::new(crate::core::recording::preview_slot::PreviewFrameSlot::default());
        self.camera_sequence = 0;
        let camera_source = Arc::clone(&self.dependencies.camera_source);
        let preview_sink: Arc<dyn MediaSink> = self.camera_preview.clone();
        log_flow_event(events::CAMERA_STARTING, serde_json::json!({"edge": edge}));
        let start_task = Task::perform(
            async move {
                tokio::task::spawn_blocking(move || camera_source.start_capture(preview_sink))
                    .await
                    .map_err(|join_error| join_error.to_string())
                    .and_then(|result| {
                        result
                            .map(CaptureHandle::wrap)
                            .map_err(|error| error.to_string())
                    })
            },
            RecordingMessage::CameraStarted,
        );
        Task::batch([open_task, start_task])
    }

    pub(super) fn stop_camera(&mut self) -> Task<RecordingMessage> {
        if let Some(capture) = self.camera_capture.take() {
            let stopped = capture.stop();
            log_flow_event(
                events::CAMERA_STOPPED,
                serde_json::json!({"stopped": stopped}),
            );
        }
        close_if_open(self.windows.camera.take())
    }

    pub(super) fn handle_camera_started(
        &mut self,
        result: Result<CaptureHandle, String>,
    ) -> Task<RecordingMessage> {
        match result {
            Ok(capture) if self.windows.camera.is_some() => {
                log_flow_event(events::CAMERA_STARTED, serde_json::json!({}));
                self.camera_capture = Some(capture);
            }
            Ok(capture) => {
                let _ = capture.stop();
            }
            Err(reason) => {
                log_flow_event(
                    events::CAMERA_START_FAILED,
                    serde_json::json!({"error": reason}),
                );
                self.camera_view.content = CameraBubbleContent::Unavailable(reason);
            }
        }
        Task::none()
    }

    pub(super) fn handle_camera_bubble_message(
        &mut self,
        message: CameraBubbleMessage,
    ) -> Task<RecordingMessage> {
        match (message, self.windows.camera) {
            (CameraBubbleMessage::DragRequested, Some(camera_window)) => {
                self.camera_user_positioned = true;
                window::drag(camera_window)
            }
            (CameraBubbleMessage::DragRequested, None) => Task::none(),
        }
    }

    pub(super) fn refresh_camera_preview(&mut self) -> bool {
        let Some((sequence, frame)) = self.camera_preview.take_newer_than(self.camera_sequence)
        else {
            return false;
        };
        self.camera_sequence = sequence;
        let pixels = Arc::try_unwrap(frame.data).unwrap_or_else(|shared| (*shared).clone());
        self.camera_view.content =
            CameraBubbleContent::Live(image::Handle::from_rgba(frame.width, frame.height, pixels));
        true
    }

    pub(super) fn reposition_camera_for_selection(&mut self) -> Task<RecordingMessage> {
        let (Some(camera_window), Some(display), Some(selection)) = (
            self.windows.camera,
            self.display,
            self.selected_capture_rectangle(),
        ) else {
            return Task::none();
        };
        if self.camera_user_positioned {
            return Task::none();
        }
        window::move_to(
            camera_window,
            display.camera_bubble_position(selection, self.camera_view.edge),
        )
    }
}
