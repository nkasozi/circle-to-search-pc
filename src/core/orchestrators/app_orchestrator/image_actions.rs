use super::*;

impl AppOrchestrator {
    pub(super) fn handle_copy_image_to_clipboard(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
        draw_strokes: Vec<crate::presentation::DrawStroke>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "clipboard_copy_started",
            serde_json::json!({"draw_strokes": draw_strokes.len()}),
        );

        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                crate::presentation::InteractiveOcrMessage::CopyImagePreparing,
            )),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::CopyImageCopying,
                )
            }),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let rgba_data = Self::build_clipboard_image_data(&buffer, &draw_strokes);
                Self::copy_image_to_clipboard_message(
                    window_id,
                    &buffer,
                    &rgba_data,
                    correlation_id,
                )
            }),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(2600)).await;
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::HideToast,
                )
            }),
        ])
    }

    pub(super) fn build_clipboard_image_data(
        buffer: &CaptureBuffer,
        draw_strokes: &[crate::presentation::DrawStroke],
    ) -> Vec<u8> {
        let rgba_data = buffer.raw_data.clone();

        if draw_strokes.is_empty() {
            return rgba_data;
        }

        let converted_strokes: Vec<_> = draw_strokes
            .iter()
            .map(|stroke| {
                let points: Vec<(f32, f32)> = stroke
                    .points
                    .iter()
                    .map(|point| (point.x, point.y))
                    .collect();
                let color = (
                    stroke.color.r,
                    stroke.color.g,
                    stroke.color.b,
                    stroke.color.a,
                );
                (points, color, stroke.width)
            })
            .collect();

        match crate::infrastructure::utils::composite_drawings_on_image(
            &rgba_data,
            buffer.width,
            buffer.height,
            &converted_strokes,
        ) {
            Ok(composited_data) => composited_data,
            Err(composite_error) => {
                log::warn!(
                    "{}",
                    serde_json::json!({
                        "event": "drawings_composite_failed",
                        "error": composite_error.to_string(),
                    })
                );
                rgba_data
            }
        }
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
                    "clipboard_copy_succeeded",
                    serde_json::json!({}),
                );
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::CopyImageSuccess,
                )
            }
            Err(copy_error) => {
                AppOrchestrator::log_error_event_for_correlation(
                    correlation_id,
                    "clipboard_copy_failed",
                    serde_json::json!({"error": copy_error.to_string()}),
                );
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::CopyImageFailed(
                        copy_error.to_string(),
                    ),
                )
            }
        }
    }

    pub(super) fn handle_save_image_to_file(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
        draw_strokes: Vec<crate::presentation::DrawStroke>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "save_image_to_file_started",
            serde_json::json!({"draw_strokes": draw_strokes.len()}),
        );

        let save_location = self.settings.screenshot_save_location.clone();
        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                crate::presentation::InteractiveOcrMessage::SaveImagePreparing,
            )),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::SaveImageSaving,
                )
            }),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let rgba_data = Self::build_clipboard_image_data(&buffer, &draw_strokes);
                Self::save_image_to_file_message(
                    window_id,
                    &buffer,
                    &rgba_data,
                    &save_location,
                    correlation_id,
                )
            }),
            Task::future(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(2600)).await;
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::HideToast,
                )
            }),
        ])
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
                    "save_image_to_file_succeeded",
                    serde_json::json!({"path": path}),
                );
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::SaveSuccess(path),
                )
            }
            Err(save_error) => {
                AppOrchestrator::log_error_event_for_correlation(
                    correlation_id,
                    "save_image_to_file_failed",
                    serde_json::json!({"error": save_error.to_string()}),
                );
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::SaveFailed(save_error.to_string()),
                )
            }
        }
    }
}
