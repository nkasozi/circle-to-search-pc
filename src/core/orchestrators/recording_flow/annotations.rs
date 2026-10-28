use std::time::Instant;

use iced::window::{self, Id};
use iced::Task;

use super::{events, log_flow_event, RecordingFlow, RecordingMessage, RecordingPhase};
use crate::presentation::recording::annotation_overlay::ScreenAnnotationMessage;
use crate::presentation::recording::controls_view::RecordingControlsMessage;

impl RecordingFlow {
    pub(super) fn handle_controls_message(
        &mut self,
        message: RecordingControlsMessage,
    ) -> Task<RecordingMessage> {
        if !matches!(message, RecordingControlsMessage::Discard) {
            self.controls_view.discard_armed = false;
        }
        match message {
            RecordingControlsMessage::DragRequested => self
                .windows
                .controls
                .map(window::drag)
                .unwrap_or_else(Task::none),
            RecordingControlsMessage::TogglePause => self.toggle_pause(),
            RecordingControlsMessage::Stop => self.request_stop(),
            RecordingControlsMessage::Discard => {
                match self.controls_view.arm_or_confirm_discard() {
                    true => self.request_discard(),
                    false => Task::none(),
                }
            }
            RecordingControlsMessage::ToggleDrawing => {
                self.set_drawing_enabled(!self.annotation_view.drawing_enabled)
            }
            RecordingControlsMessage::SelectTool(tool) => {
                self.annotation_view.board.tool = tool;
                self.controls_view.selected_tool = tool;
                Task::none()
            }
            RecordingControlsMessage::SelectColor(color) => {
                self.annotation_view.board.color = color;
                self.controls_view.selected_color = color;
                Task::none()
            }
            RecordingControlsMessage::Undo => self.edit_board(|board| board.undo()),
            RecordingControlsMessage::ClearDrawings => self.edit_board(|board| board.clear()),
        }
    }

    fn toggle_pause(&mut self) -> Task<RecordingMessage> {
        let RecordingPhase::Recording { session } = &self.phase else {
            return Task::none();
        };
        let now_paused = session.with_session(|active_session| match active_session.is_paused() {
            true => !active_session.resume(),
            false => active_session.pause(),
        });
        log_flow_event(
            events::RECORDING_PAUSE_TOGGLED,
            serde_json::json!({"paused": now_paused}),
        );
        self.update_session_status()
    }

    fn set_drawing_enabled(&mut self, enabled: bool) -> Task<RecordingMessage> {
        let Some(annotation_window) = self.windows.annotation else {
            return Task::none();
        };
        self.annotation_view.drawing_enabled = enabled;
        self.controls_view.drawing_enabled = enabled;
        log_flow_event(
            events::DRAWING_MODE_CHANGED,
            serde_json::json!({"enabled": enabled}),
        );
        match enabled {
            true => window::disable_mouse_passthrough(annotation_window)
                .chain(window::gain_focus(annotation_window)),
            false => window::enable_mouse_passthrough(annotation_window),
        }
    }

    fn edit_board(
        &mut self,
        edit: impl FnOnce(
            crate::presentation::recording::annotation::AnnotationBoard,
        ) -> crate::presentation::recording::annotation::AnnotationBoard,
    ) -> Task<RecordingMessage> {
        self.annotation_view.board = edit(self.annotation_view.board.clone());
        self.controls_view.has_drawings = !self.annotation_view.board.strokes.is_empty();
        Task::none()
    }

    pub(super) fn handle_annotation_message(
        &mut self,
        message: ScreenAnnotationMessage,
    ) -> Task<RecordingMessage> {
        match message {
            ScreenAnnotationMessage::StrokeStarted(point) => {
                self.edit_board(|board| board.start_stroke(point))
            }
            ScreenAnnotationMessage::StrokeMoved(point) => {
                self.edit_board(|board| board.extend_stroke(point))
            }
            ScreenAnnotationMessage::StrokeFinished => {
                self.edit_board(|board| board.finish_stroke())
            }
        }
    }

    pub(super) fn handle_frame_tick(&mut self) -> Task<RecordingMessage> {
        let _ = self.refresh_camera_preview();
        self.sample_pointer();
        self.update_session_status()
    }

    fn sample_pointer(&mut self) -> bool {
        let (Some(display), Some(_)) = (self.display, self.windows.annotation) else {
            return false;
        };
        let Ok(pointer) = self.dependencies.pointer_provider.current_pointer_state() else {
            return false;
        };
        let local_position = display.to_local(pointer.x, pointer.y);
        self.annotation_view.pointer = self.annotation_view.pointer.clone().apply_sample(
            local_position,
            pointer.is_primary_button_pressed,
            Instant::now(),
        );
        true
    }

    pub(super) fn handle_escape(&mut self, window_id: Id) -> Task<RecordingMessage> {
        let is_setup_window = self.windows.setup == Some(window_id);
        let is_drawing_window =
            self.windows.annotation == Some(window_id) && self.annotation_view.drawing_enabled;
        match (is_setup_window, is_drawing_window) {
            (true, _)
                if matches!(
                    self.phase,
                    RecordingPhase::Setup | RecordingPhase::Countdown { .. }
                ) =>
            {
                self.cancel_setup()
            }
            (_, true) => self.set_drawing_enabled(false),
            _ => Task::none(),
        }
    }

    pub(super) fn handle_window_closed(&mut self, window_id: Id) -> Task<RecordingMessage> {
        let windows = &mut self.windows;
        for slot in [
            &mut windows.setup,
            &mut windows.camera,
            &mut windows.controls,
            &mut windows.annotation,
            &mut windows.ready,
        ] {
            if *slot == Some(window_id) {
                *slot = None;
            }
        }
        if self.windows.ready.is_none() {
            self.ready_view = None;
        }
        Task::none()
    }
}
