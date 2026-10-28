use super::super::*;
use super::constants::*;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_windows_list_loaded(
        &mut self,
        window_id: Id,
        windows: Vec<WindowInfo>,
    ) -> Task<OrchestratorMessage> {
        self.log_debug_event(
            EVENT_WINDOW_PICKER_WINDOWS_LIST_APPLIED,
            serde_json::json!({"window_count": windows.len()}),
        );

        if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
            view.set_windows(windows);
        }

        Task::none()
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_focus_window_and_capture(
        &mut self,
        app_name: String,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_FOCUS_WINDOW_AND_CAPTURE_STARTED,
            serde_json::json!({
                "app_name": app_name,
            }),
        );

        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            let _ = crate::infrastructure::utils::focus_external_window_by_app_name(&app_name);
            tokio::time::sleep(std::time::Duration::from_millis(
                FOCUS_WINDOW_SETTLE_DELAY_MILLISECONDS,
            ))
            .await;
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id,
                EVENT_FOCUS_WINDOW_AND_CAPTURE_COMPLETED,
                serde_json::json!({}),
            );
            OrchestratorMessage::CaptureScreen
        })
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_capture_selected_window(
        &mut self,
        window_id: u32,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_WINDOW_CAPTURE_SELECTED_STARTED,
            serde_json::json!({
                "selected_window_id": window_id,
            }),
        );
        self.window_picker_window_id = None;

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            build_selected_window_capture_message(
                screen_capturer.as_ref(),
                window_id,
                correlation_id,
            )
        })
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_window_capture_complete(
        &mut self,
        capture_buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_WINDOW_CAPTURE_PROCESSING,
            serde_json::json!({
                "width": capture_buffer.width,
                "height": capture_buffer.height,
            }),
        );

        let selection_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: capture_buffer.width as f32,
            height: capture_buffer.height as f32,
        };

        self.update(OrchestratorMessage::ShowCroppedImage(
            capture_buffer,
            selection_rect,
        ))
    }
}

fn build_selected_window_capture_message(
    screen_capturer: &dyn ScreenCapturer,
    selected_window_id: u32,
    correlation_id: String,
) -> OrchestratorMessage {
    match screen_capturer.capture_window_by_id(selected_window_id) {
        Ok(capture_buffer) => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id,
                EVENT_WINDOW_CAPTURE_SELECTED_COMPLETED,
                serde_json::json!({
                    "selected_window_id": selected_window_id,
                    "width": capture_buffer.width,
                    "height": capture_buffer.height,
                }),
            );
            OrchestratorMessage::WindowCaptureComplete(capture_buffer)
        }
        Err(error) => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id,
                EVENT_WINDOW_CAPTURE_SELECTED_FAILED,
                serde_json::json!({
                    "selected_window_id": selected_window_id,
                    "error": error.to_string(),
                }),
            );
            OrchestratorMessage::WindowCaptureError(error.to_string())
        }
    }
}
