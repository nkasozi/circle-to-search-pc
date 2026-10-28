use super::super::*;
use super::constants::*;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_open_capture_overlay(
        &mut self,
        mouse_x: i32,
        mouse_y: i32,
        capture_buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_CAPTURE_OVERLAY_OPENING,
            serde_json::json!({"mouse_x": mouse_x, "mouse_y": mouse_y}),
        );
        let monitor = match xcap::Monitor::from_point(mouse_x, mouse_y) {
            Ok(monitor) => monitor,
            Err(monitor_error) => {
                self.log_error_event(
                    EVENT_CAPTURE_OVERLAY_MONITOR_FAILED,
                    serde_json::json!({"error": monitor_error.to_string()}),
                );
                self.status = format!(
                    "{}{}",
                    global_constants::CAPTURE_ERROR_MONITOR_PREFIX,
                    monitor_error
                );
                return Task::none();
            }
        };
        self.log_info_event(EVENT_CAPTURE_OVERLAY_MONITOR_FOUND, serde_json::json!({}));
        self.open_capture_overlay_on_monitor(&monitor, capture_buffer)
    }

    fn open_capture_overlay_on_monitor(
        &mut self,
        monitor: &xcap::Monitor,
        capture_buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        let (overlay_window_id, open_task) = window::open(build_overlay_window_settings(monitor));

        let capture_view = CaptureView::build_with_capture_buffer(capture_buffer);
        self.windows
            .insert(overlay_window_id, AppWindow::CaptureOverlay(capture_view));
        self.status = global_constants::STATUS_OVERLAY_READY.to_string();
        self.log_info_event(
            EVENT_CAPTURE_OVERLAY_CREATED,
            serde_json::json!({"window_id": format!("{:?}", overlay_window_id)}),
        );

        open_task
            .discard()
            .chain(window::gain_focus(overlay_window_id))
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_escape_pressed(
        &mut self,
        window_id: Id,
    ) -> Task<OrchestratorMessage> {
        if self.recording_flow.owns_window(window_id) {
            return self.handle_recording_message(RecordingMessage::EscapePressed(window_id));
        }
        let is_dismissable_window = matches!(
            self.windows.get(&window_id),
            Some(AppWindow::CaptureOverlay(_))
                | Some(AppWindow::WindowPicker(_))
                | Some(AppWindow::Player(_))
        );
        self.log_info_event(
            EVENT_ESCAPE_PRESSED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "is_dismissable_window": is_dismissable_window,
            }),
        );
        if !is_dismissable_window {
            return Task::none();
        }
        self.status = global_constants::STATUS_READY.to_string();
        window::close(window_id)
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_global_hotkey(
        &mut self,
        action: GlobalHotkeyAction,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_GLOBAL_HOTKEY_PRESSED,
            serde_json::json!({"action": format!("{:?}", action)}),
        );
        match action {
            GlobalHotkeyAction::Capture => self.handle_capture_screen(),
            GlobalHotkeyAction::ToggleRecording => self.handle_toggle_recording_hotkey(),
        }
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_capture_overlay_message(
        &mut self,
        window_id: Id,
        capture_message: CaptureViewMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_CAPTURE_OVERLAY_MESSAGE_RECEIVED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "message": format!("{:?}", capture_message),
            }),
        );
        match capture_message {
            CaptureViewMessage::ConfirmSelection => {
                self.update(OrchestratorMessage::ConfirmSelection(window_id))
            }
            CaptureViewMessage::SwitchToRecording => self.switch_overlay_to_recording(window_id),
            CaptureViewMessage::SelectWindow => self.switch_overlay_to_window_picker(window_id),
            other_message => self.forward_message_to_capture_overlay(window_id, other_message),
        }
    }

    fn switch_overlay_to_recording(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_CAPTURE_OVERLAY_SWITCHING_TO_RECORDING,
            serde_json::json!({}),
        );
        Task::batch(vec![
            window::close(window_id),
            self.handle_recording_message(RecordingMessage::Toggle),
        ])
    }

    fn switch_overlay_to_window_picker(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_WINDOW_SELECTION_REQUESTED_FROM_OVERLAY,
            serde_json::json!({}),
        );
        Task::batch(vec![
            window::close(window_id),
            Task::done(OrchestratorMessage::OpenWindowPicker),
        ])
    }

    fn forward_message_to_capture_overlay(
        &mut self,
        window_id: Id,
        capture_message: CaptureViewMessage,
    ) -> Task<OrchestratorMessage> {
        let is_release = matches!(capture_message, CaptureViewMessage::MouseReleased);
        let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get_mut(&window_id) else {
            self.log_error_event(
                EVENT_CAPTURE_OVERLAY_WINDOW_NOT_FOUND,
                serde_json::json!({"window_id": format!("{:?}", window_id)}),
            );
            return Task::none();
        };
        capture_view.update(capture_message);
        let should_confirm_selection = is_release && capture_view.has_completed_selection();
        match should_confirm_selection {
            true => self.update(OrchestratorMessage::ConfirmSelection(window_id)),
            false => Task::none(),
        }
    }
}

fn build_overlay_window_settings(monitor: &xcap::Monitor) -> window::Settings {
    window::Settings {
        position: window::Position::Specific(Point::new(
            monitor.x().unwrap_or(MONITOR_FALLBACK_ORIGIN) as f32,
            monitor.y().unwrap_or(MONITOR_FALLBACK_ORIGIN) as f32,
        )),
        size: Size::new(
            monitor.width().unwrap_or(MONITOR_FALLBACK_WIDTH) as f32,
            monitor.height().unwrap_or(MONITOR_FALLBACK_HEIGHT) as f32,
        ),
        transparent: true,
        decorations: false,
        ..Default::default()
    }
}
