use super::super::*;
use super::constants::*;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_process_ocr(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_OCR_PROCESSING_STARTED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "buffer_width": buffer.width,
                "buffer_height": buffer.height,
            }),
        );

        let ocr_service = self.ocr_service.clone();
        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            let ocr_outcome = run_ocr_on_buffer(ocr_service, buffer, &correlation_id).await;
            OrchestratorMessage::OcrComplete(window_id, ocr_outcome)
        })
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_ocr_complete(
        &mut self,
        window_id: Id,
        result: Result<OcrResult, String>,
    ) -> Task<OrchestratorMessage> {
        match result {
            Ok(ocr_result) => self.apply_ocr_success(window_id, ocr_result),
            Err(ocr_error) => self.apply_ocr_failure(window_id, ocr_error),
        }
    }

    fn apply_ocr_success(
        &mut self,
        window_id: Id,
        ocr_result: OcrResult,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_OCR_COMPLETE,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "text_blocks": ocr_result.text_blocks.len(),
            }),
        );

        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) else {
            return Task::none();
        };
        view.set_ocr_result(ocr_result);
        self.status = global_constants::STATUS_OCR_COMPLETE.to_string();
        Task::none()
    }

    fn apply_ocr_failure(&mut self, window_id: Id, ocr_error: String) -> Task<OrchestratorMessage> {
        self.log_error_event(
            EVENT_OCR_FAILED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "error": ocr_error,
            }),
        );
        self.status = global_constants::STATUS_READY.to_string();
        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) else {
            return Task::none();
        };
        view.set_ocr_failed(ocr_error);
        Task::none()
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_ocr_service_ready(
        &mut self,
        service: Arc<dyn OcrService>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(EVENT_OCR_SERVICE_READY, serde_json::json!({}));
        self.ocr_service = service;
        self.status = global_constants::STATUS_READY.to_string();
        Task::none()
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_ocr_service_failed(
        &mut self,
        error: String,
    ) -> Task<OrchestratorMessage> {
        self.log_error_event(
            EVENT_OCR_SERVICE_INITIALIZATION_FAILED,
            serde_json::json!({"error": error}),
        );
        self.status = format!(
            "{}{}",
            global_constants::OCR_INITIALIZATION_FAILED_PREFIX,
            error
        );
        Task::none()
    }
}

async fn run_ocr_on_buffer(
    ocr_service: Arc<dyn OcrService>,
    buffer: CaptureBuffer,
    correlation_id: &str,
) -> Result<OcrResult, String> {
    let dynamic_image = convert_buffer_to_dynamic_image(buffer, correlation_id)?;
    AppOrchestrator::log_info_event_for_correlation(
        correlation_id.to_string(),
        EVENT_OCR_RUNNING,
        serde_json::json!({}),
    );
    match ocr_service.extract_text_from_image(&dynamic_image).await {
        Ok(result) => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id.to_string(),
                EVENT_OCR_COMPLETED,
                serde_json::json!({"text_blocks": result.text_blocks.len()}),
            );
            Ok(result)
        }
        Err(ocr_error) => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id.to_string(),
                EVENT_OCR_FAILED,
                serde_json::json!({"error": ocr_error.to_string()}),
            );
            Err(ocr_error.to_string())
        }
    }
}

fn convert_buffer_to_dynamic_image(
    buffer: CaptureBuffer,
    correlation_id: &str,
) -> Result<image::DynamicImage, String> {
    let width = buffer.width;
    let height = buffer.height;
    AppOrchestrator::log_info_event_for_correlation(
        correlation_id.to_string(),
        EVENT_OCR_IMAGE_CONVERTING,
        serde_json::json!({"width": width, "height": height}),
    );

    let Some(rgba_image) = image::RgbaImage::from_raw(width, height, buffer.raw_data) else {
        AppOrchestrator::log_error_event_for_correlation(
            correlation_id.to_string(),
            EVENT_OCR_IMAGE_CONVERSION_FAILED,
            serde_json::json!({"width": width, "height": height}),
        );
        return Err(global_constants::OCR_RAW_IMAGE_CREATION_FAILED.to_string());
    };
    Ok(image::DynamicImage::ImageRgba8(rgba_image))
}
