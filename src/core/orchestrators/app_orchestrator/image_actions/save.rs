use super::super::*;
use super::build_delayed_interactive_ocr_message_task;
use super::constants::*;
use crate::presentation::{DrawStroke, InteractiveOcrMessage};

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_save_image_to_file(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
        draw_strokes: Vec<DrawStroke>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_SAVE_IMAGE_TO_FILE_STARTED,
            serde_json::json!({"draw_strokes": draw_strokes.len()}),
        );

        let save_location = self.settings.screenshot_save_location.clone();
        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::SaveImagePreparing,
            )),
            build_delayed_interactive_ocr_message_task(
                window_id,
                PROGRESS_STEP_DELAY_MILLISECONDS,
                InteractiveOcrMessage::SaveImageSaving,
            ),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    IMAGE_WORK_DELAY_MILLISECONDS,
                ))
                .await;
                let rgba_data = Self::build_clipboard_image_data(&buffer, &draw_strokes);
                save_image_to_file_message(
                    window_id,
                    &buffer,
                    &rgba_data,
                    &save_location,
                    correlation_id,
                )
            }),
            build_delayed_interactive_ocr_message_task(
                window_id,
                TOAST_HIDE_DELAY_MILLISECONDS,
                InteractiveOcrMessage::HideToast,
            ),
        ])
    }
}

fn save_image_to_file_message(
    window_id: Id,
    buffer: &CaptureBuffer,
    rgba_data: &[u8],
    save_location: &str,
    correlation_id: String,
) -> OrchestratorMessage {
    match crate::infrastructure::utils::save_image_to_file(
        rgba_data,
        buffer.width,
        buffer.height,
        save_location,
    ) {
        Ok(path) => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id,
                EVENT_SAVE_IMAGE_TO_FILE_SUCCEEDED,
                serde_json::json!({"path": path}),
            );
            OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::SaveSuccess(path),
            )
        }
        Err(save_error) => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id,
                EVENT_SAVE_IMAGE_TO_FILE_FAILED,
                serde_json::json!({"error": save_error.to_string()}),
            );
            OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::SaveFailed(save_error.to_string()),
            )
        }
    }
}
