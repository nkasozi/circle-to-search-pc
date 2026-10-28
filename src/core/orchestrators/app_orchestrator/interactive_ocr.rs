use super::*;

mod constants;
mod image_search;
mod ocr_processing;

use crate::presentation::InteractiveOcrMessage;
use constants::*;

impl AppOrchestrator {
    pub(super) fn handle_interactive_ocr_message(
        &mut self,
        window_id: Id,
        ocr_message: InteractiveOcrMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_INTERACTIVE_OCR_MESSAGE_RECEIVED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "message": format!("{:?}", ocr_message),
            }),
        );

        if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
            view.update(ocr_message.clone());
        }

        self.route_interactive_ocr_message(window_id, ocr_message)
    }

    fn route_interactive_ocr_message(
        &mut self,
        window_id: Id,
        ocr_message: InteractiveOcrMessage,
    ) -> Task<OrchestratorMessage> {
        match ocr_message {
            InteractiveOcrMessage::Close => window::close(window_id),
            InteractiveOcrMessage::SearchSelected => self.start_selected_image_search(window_id),
            InteractiveOcrMessage::CopySelected => build_delayed_hide_toast_task(window_id),
            InteractiveOcrMessage::CopyImageToClipboard => self.start_copy_image(window_id),
            InteractiveOcrMessage::SaveImageToFile => self.start_save_image(window_id),
            InteractiveOcrMessage::Recrop => self.start_recrop(window_id),
            InteractiveOcrMessage::StartOcr => self.start_ocr_processing(window_id),
            InteractiveOcrMessage::CancelOcr => {
                self.log_info_event(
                    EVENT_OCR_CANCELLED_BY_USER,
                    serde_json::json!({"window_id": format!("{:?}", window_id)}),
                );
                Task::none()
            }
            InteractiveOcrMessage::RetryOcr => self.start_ocr_processing(window_id),
            _ => Task::none(),
        }
    }

    fn start_selected_image_search(&self, window_id: Id) -> Task<OrchestratorMessage> {
        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) else {
            return Task::none();
        };
        let buffer = view.get_capture_buffer().clone();
        let query = view.get_search_query().to_string();
        let query_option = match query.is_empty() {
            true => None,
            false => Some(query),
        };
        Task::done(OrchestratorMessage::PerformImageSearch(
            window_id,
            buffer,
            query_option,
        ))
    }

    fn start_copy_image(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) else {
            return Task::none();
        };
        let buffer = view.get_capture_buffer().clone();
        let draw_strokes = view.get_draw_strokes();
        self.update(OrchestratorMessage::CopyImageToClipboard(
            window_id,
            buffer,
            draw_strokes,
        ))
    }

    fn start_save_image(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) else {
            return Task::none();
        };
        let buffer = view.get_capture_buffer().clone();
        let draw_strokes = view.get_draw_strokes();
        self.update(OrchestratorMessage::SaveImageToFile(
            window_id,
            buffer,
            draw_strokes,
        ))
    }

    fn start_recrop(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) else {
            return Task::none();
        };
        self.pending_draw_strokes = Some(view.get_draw_strokes());
        Task::batch(vec![
            window::close(window_id),
            self.update(OrchestratorMessage::CaptureScreen),
        ])
    }

    fn start_ocr_processing(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_OCR_PROCESSING_REQUESTED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
            }),
        );

        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) else {
            return Task::none();
        };
        let buffer = view.get_capture_buffer().clone();
        self.status = global_constants::STATUS_PROCESSING_OCR.to_string();
        Task::done(OrchestratorMessage::ProcessOcr(window_id, buffer))
    }
}

fn build_delayed_hide_toast_task(window_id: Id) -> Task<OrchestratorMessage> {
    Task::future(async move {
        tokio::time::sleep(std::time::Duration::from_secs(
            COPY_TOAST_HIDE_DELAY_SECONDS,
        ))
        .await;
        OrchestratorMessage::InteractiveOcrMessage(window_id, InteractiveOcrMessage::HideToast)
    })
}
