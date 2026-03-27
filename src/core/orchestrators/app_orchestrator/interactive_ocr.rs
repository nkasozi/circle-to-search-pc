use super::*;

impl AppOrchestrator {
    pub(super) fn handle_interactive_ocr_message(
        &mut self,
        window_id: Id,
        ocr_msg: crate::presentation::InteractiveOcrMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "interactive_ocr_message_received",
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "message": format!("{:?}", ocr_msg),
            }),
        );

        if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
            view.update(ocr_msg.clone());
        }

        match ocr_msg {
            crate::presentation::InteractiveOcrMessage::Close => window::close(window_id),
            crate::presentation::InteractiveOcrMessage::SearchSelected => {
                self.start_selected_image_search(window_id)
            }
            crate::presentation::InteractiveOcrMessage::CopySelected => Task::future(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                OrchestratorMessage::InteractiveOcrMessage(
                    window_id,
                    crate::presentation::InteractiveOcrMessage::HideToast,
                )
            }),
            crate::presentation::InteractiveOcrMessage::CopyImageToClipboard => {
                self.start_copy_image(window_id)
            }
            crate::presentation::InteractiveOcrMessage::SaveImageToFile => {
                self.start_save_image(window_id)
            }
            crate::presentation::InteractiveOcrMessage::Recrop => self.start_recrop(window_id),
            crate::presentation::InteractiveOcrMessage::StartOcr => {
                self.start_ocr_processing(window_id)
            }
            crate::presentation::InteractiveOcrMessage::CancelOcr => {
                log::info!(
                    "[ORCHESTRATOR] OCR cancelled by user for window {:?}",
                    window_id
                );
                window::close(window_id)
            }
            crate::presentation::InteractiveOcrMessage::RetryOcr => {
                self.start_ocr_processing(window_id)
            }
            _ => Task::none(),
        }
    }

    fn start_selected_image_search(&self, window_id: Id) -> Task<OrchestratorMessage> {
        let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) else {
            return Task::none();
        };
        let buffer = view.get_capture_buffer().clone();
        let query = view.get_search_query().to_string();
        let query_option = if query.is_empty() { None } else { Some(query) };
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
            "ocr_processing_requested",
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

    pub(super) fn handle_perform_image_search(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
        query: Option<String>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "image_search_started",
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "has_query": query.as_ref().map(|q| !q.is_empty()).unwrap_or(false),
            }),
        );

        let search_provider = Arc::clone(&self.reverse_image_search_provider);
        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                crate::presentation::InteractiveOcrMessage::SearchUploading,
            )),
            Task::future(async move {
                let search_future = search_provider.perform_search(&buffer, query.as_deref());
                let timeout_duration =
                    std::time::Duration::from_secs(global_constants::IMAGE_SEARCH_TIMEOUT_SECONDS);

                match tokio::time::timeout(timeout_duration, search_future).await {
                    Ok(Ok(_search_url)) => {
                        AppOrchestrator::log_info_event_for_correlation(
                            correlation_id.clone(),
                            "image_search_completed",
                            serde_json::json!({
                                "window_id": format!("{:?}", window_id),
                            }),
                        );
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchCompleted,
                        )
                    }
                    Ok(Err(error)) => {
                        AppOrchestrator::log_error_event_for_correlation(
                            correlation_id.clone(),
                            "image_search_failed",
                            serde_json::json!({
                                "window_id": format!("{:?}", window_id),
                                "error": error.to_string(),
                            }),
                        );
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchFailed(format!(
                                "{}{}",
                                error,
                                global_constants::IMAGE_SEARCH_FAILURE_SUFFIX
                            )),
                        )
                    }
                    Err(_) => {
                        AppOrchestrator::log_error_event_for_correlation(
                            correlation_id.clone(),
                            "image_search_timeout",
                            serde_json::json!({
                                "window_id": format!("{:?}", window_id),
                                "timeout_seconds": global_constants::IMAGE_SEARCH_TIMEOUT_SECONDS,
                            }),
                        );
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchFailed(
                                global_constants::IMAGE_SEARCH_TIMEOUT_MESSAGE.to_string(),
                            ),
                        )
                    }
                }
            }),
        ])
    }

    pub(super) fn handle_process_ocr(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "ocr_processing_started",
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "buffer_width": buffer.width,
                "buffer_height": buffer.height,
            }),
        );

        let ocr_service = self.ocr_service.clone();
        let width = buffer.width;
        let height = buffer.height;
        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id.clone(),
                "ocr_image_converting",
                serde_json::json!({"width": width, "height": height}),
            );

            let raw_image = image::RgbaImage::from_raw(width, height, buffer.raw_data.clone());

            match raw_image {
                None => {
                    AppOrchestrator::log_error_event_for_correlation(
                        correlation_id.clone(),
                        "ocr_image_conversion_failed",
                        serde_json::json!({"width": width, "height": height}),
                    );
                    OrchestratorMessage::OcrComplete(
                        window_id,
                        Err(global_constants::OCR_RAW_IMAGE_CREATION_FAILED.to_string()),
                    )
                }
                Some(rgba_image) => {
                    let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);
                    AppOrchestrator::log_info_event_for_correlation(
                        correlation_id.clone(),
                        "ocr_running",
                        serde_json::json!({}),
                    );
                    match ocr_service.extract_text_from_image(&dynamic_image).await {
                        Ok(result) => {
                            AppOrchestrator::log_info_event_for_correlation(
                                correlation_id.clone(),
                                "ocr_completed",
                                serde_json::json!({"text_blocks": result.text_blocks.len()}),
                            );
                            OrchestratorMessage::OcrComplete(window_id, Ok(result))
                        }
                        Err(ocr_error) => {
                            AppOrchestrator::log_error_event_for_correlation(
                                correlation_id.clone(),
                                "ocr_failed",
                                serde_json::json!({"error": ocr_error.to_string()}),
                            );
                            OrchestratorMessage::OcrComplete(window_id, Err(ocr_error.to_string()))
                        }
                    }
                }
            }
        })
    }

    pub(super) fn handle_ocr_complete(
        &mut self,
        window_id: Id,
        result: Result<OcrResult, String>,
    ) -> Task<OrchestratorMessage> {
        match result {
            Ok(ocr_result) => {
                self.log_info_event(
                    "ocr_complete",
                    serde_json::json!({
                        "window_id": format!("{:?}", window_id),
                        "text_blocks": ocr_result.text_blocks.len(),
                    }),
                );

                if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
                    view.set_ocr_result(ocr_result);
                    self.status = global_constants::STATUS_OCR_COMPLETE.to_string();
                }
            }
            Err(ocr_error) => {
                self.log_error_event(
                    "ocr_failed",
                    serde_json::json!({
                        "window_id": format!("{:?}", window_id),
                        "error": ocr_error,
                    }),
                );
                self.status = global_constants::STATUS_READY.to_string();
                if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
                    view.set_ocr_failed(ocr_error);
                }
            }
        }
        Task::none()
    }

    pub(super) fn handle_ocr_service_ready(
        &mut self,
        service: Arc<dyn OcrService>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event("ocr_service_ready", serde_json::json!({}));
        self.ocr_service = service;
        self.status = global_constants::STATUS_READY.to_string();
        Task::none()
    }

    pub(super) fn handle_ocr_service_failed(&mut self, error: String) -> Task<OrchestratorMessage> {
        self.log_error_event(
            "ocr_service_initialization_failed",
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
