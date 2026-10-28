use super::super::*;
use super::constants::*;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_confirm_selection(
        &mut self,
        overlay_id: Id,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_SELECTION_CONFIRMING,
            serde_json::json!({"overlay_id": format!("{:?}", overlay_id)}),
        );

        let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get(&overlay_id) else {
            self.log_error_event(
                EVENT_CONFIRM_SELECTION_OVERLAY_NOT_FOUND,
                serde_json::json!({}),
            );
            return window::close(overlay_id);
        };

        let Some(selection_rect) = capture_view.get_selected_region() else {
            self.log_error_event(EVENT_CONFIRM_SELECTION_NO_REGION, serde_json::json!({}));
            return window::close(overlay_id);
        };

        self.log_info_event(
            EVENT_SELECTION_CONFIRMED,
            serde_json::json!({"rect": format!("{:?}", selection_rect)}),
        );
        let capture_buffer = capture_view.get_capture_buffer().clone();

        self.status = global_constants::STATUS_PROCESSING_SELECTION.to_string();
        Task::batch(vec![
            window::close(overlay_id),
            Task::done(OrchestratorMessage::ShowCroppedImage(
                capture_buffer,
                selection_rect,
            )),
        ])
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_show_cropped_image(
        &mut self,
        capture_buffer: CaptureBuffer,
        selection_rect: Rectangle,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_CROPPED_IMAGE_SHOWING,
            serde_json::json!({"rect": format!("{:?}", selection_rect)}),
        );

        let cropped_buffer = capture_buffer.crop_region(
            selection_rect.x as u32,
            selection_rect.y as u32,
            selection_rect.width as u32,
            selection_rect.height as u32,
        );

        match cropped_buffer {
            Ok(buffer) => self.open_interactive_ocr_window(buffer),
            Err(crop_error) => {
                self.log_error_event(
                    EVENT_IMAGE_CROP_FAILED,
                    serde_json::json!({"error": crop_error.to_string()}),
                );
                self.status = format!(
                    "{}{}",
                    global_constants::CAPTURE_ERROR_CROP_PREFIX,
                    crop_error
                );
                Task::none()
            }
        }
    }

    fn open_interactive_ocr_window(&mut self, buffer: CaptureBuffer) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_IMAGE_CROPPED,
            serde_json::json!({"width": buffer.width, "height": buffer.height}),
        );

        let (ocr_window_id, open_task) = window::open(build_cropped_image_window_settings(&buffer));

        let mut view = crate::presentation::InteractiveOcrView::build(
            buffer.clone(),
            self.settings.theme_mode.clone(),
        );

        if let Some(strokes) = self.pending_draw_strokes.take() {
            view.set_draw_strokes(strokes);
        }

        self.windows
            .insert(ocr_window_id, AppWindow::InteractiveOcr(view));
        self.status = global_constants::STATUS_READY_SIMPLE.to_string();

        open_task
            .discard()
            .chain(Task::done(OrchestratorMessage::InteractiveOcrMessage(
                ocr_window_id,
                crate::presentation::InteractiveOcrMessage::StartOcr,
            )))
    }
}

fn build_cropped_image_window_settings(buffer: &CaptureBuffer) -> window::Settings {
    window::Settings {
        size: Size::new(
            (buffer.width as f32).min(CROPPED_IMAGE_WINDOW_MAX_WIDTH),
            (buffer.height as f32).min(CROPPED_IMAGE_WINDOW_MAX_HEIGHT),
        ),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    }
}
