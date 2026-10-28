use super::*;

mod composite;
mod constants;
mod save;

use crate::presentation::{DrawStroke, InteractiveOcrMessage};
use constants::*;

impl AppOrchestrator {
    pub(super) fn handle_copy_image_to_clipboard(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
        draw_strokes: Vec<DrawStroke>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_CLIPBOARD_COPY_STARTED,
            serde_json::json!({"draw_strokes": draw_strokes.len()}),
        );

        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::CopyImagePreparing,
            )),
            build_delayed_interactive_ocr_message_task(
                window_id,
                PROGRESS_STEP_DELAY_MILLISECONDS,
                InteractiveOcrMessage::CopyImageCopying,
            ),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    IMAGE_WORK_DELAY_MILLISECONDS,
                ))
                .await;
                let rgba_data = Self::build_clipboard_image_data(&buffer, &draw_strokes);
                copy_image_to_clipboard_message(window_id, &buffer, &rgba_data, correlation_id)
            }),
            build_delayed_interactive_ocr_message_task(
                window_id,
                TOAST_HIDE_DELAY_MILLISECONDS,
                InteractiveOcrMessage::HideToast,
            ),
        ])
    }
}

fn build_delayed_interactive_ocr_message_task(
    window_id: Id,
    delay_milliseconds: u64,
    ocr_message: InteractiveOcrMessage,
) -> Task<OrchestratorMessage> {
    Task::future(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_milliseconds)).await;
        OrchestratorMessage::InteractiveOcrMessage(window_id, ocr_message)
    })
}

fn copy_image_to_clipboard_message(
    window_id: Id,
    buffer: &CaptureBuffer,
    rgba_data: &[u8],
    correlation_id: String,
) -> OrchestratorMessage {
    match crate::infrastructure::utils::copy_image_to_clipboard(
        rgba_data,
        buffer.width,
        buffer.height,
    ) {
        Ok(()) => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id,
                EVENT_CLIPBOARD_COPY_SUCCEEDED,
                serde_json::json!({}),
            );
            OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::CopyImageSuccess,
            )
        }
        Err(copy_error) => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id,
                EVENT_CLIPBOARD_COPY_FAILED,
                serde_json::json!({"error": copy_error.to_string()}),
            );
            OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::CopyImageFailed(copy_error.to_string()),
            )
        }
    }
}
