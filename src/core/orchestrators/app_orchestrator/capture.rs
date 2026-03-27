use super::*;

impl AppOrchestrator {
    pub(super) fn handle_capture_screen(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "capture_screen_started",
            serde_json::json!({
                "status": global_constants::STATUS_PREPARING_CAPTURE,
            }),
        );
        self.status = global_constants::STATUS_PREPARING_CAPTURE.to_string();

        let main_window_id = self.main_window_id;

        self.log_info_event(
            "capture_screen_minimizing_window",
            serde_json::json!({
                "has_main_window": main_window_id.is_some(),
            }),
        );
        Task::batch(vec![
            if let Some(id) = main_window_id {
                window::minimize(id, true)
            } else {
                Task::none()
            },
            Task::future(async {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                OrchestratorMessage::PerformCapture
            }),
        ])
    }

    pub(super) fn handle_perform_capture(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("screen_capture_performing", serde_json::json!({}));
        self.status = global_constants::STATUS_CAPTURING_SCREEN.to_string();

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            let (mouse_x, mouse_y) = match Mouse::get_mouse_position() {
                Mouse::Position { x, y } => {
                    AppOrchestrator::log_info_event_for_correlation(
                        correlation_id.clone(),
                        "mouse_position_resolved",
                        serde_json::json!({"x": x, "y": y}),
                    );
                    (x, y)
                }
                Mouse::Error => {
                    AppOrchestrator::log_error_event_for_correlation(
                        correlation_id.clone(),
                        "mouse_position_failed",
                        serde_json::json!({"fallback": "0,0"}),
                    );
                    (0, 0)
                }
            };

            let region = ScreenRegion::at_coordinates(mouse_x, mouse_y);

            match screen_capturer.capture_screen_at_region(&region) {
                Ok(capture_buffer) => {
                    AppOrchestrator::log_info_event_for_correlation(
                        correlation_id.clone(),
                        "screen_captured",
                        serde_json::json!({
                            "width": capture_buffer.width,
                            "height": capture_buffer.height,
                        }),
                    );
                    OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer)
                }
                Err(capture_error) => {
                    AppOrchestrator::log_error_event_for_correlation(
                        correlation_id.clone(),
                        "screen_capture_failed",
                        serde_json::json!({"error": capture_error.to_string()}),
                    );
                    OrchestratorMessage::CaptureError(format!(
                        "{}{}{}",
                        global_constants::CAPTURE_ERROR_GENERIC_PREFIX,
                        capture_error,
                        global_constants::CAPTURE_ERROR_GENERIC_SUFFIX
                    ))
                }
            }
        })
    }

    pub(super) fn handle_open_capture_overlay(
        &mut self,
        mouse_x: i32,
        mouse_y: i32,
        capture_buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "capture_overlay_opening",
            serde_json::json!({"mouse_x": mouse_x, "mouse_y": mouse_y}),
        );
        match xcap::Monitor::from_point(mouse_x, mouse_y) {
            Ok(monitor) => {
                self.log_info_event("capture_overlay_monitor_found", serde_json::json!({}));
                let (id, task) = window::open(window::Settings {
                    position: window::Position::Specific(Point::new(
                        monitor.x().unwrap_or(0) as f32,
                        monitor.y().unwrap_or(0) as f32,
                    )),
                    size: Size::new(
                        monitor.width().unwrap_or(1920) as f32,
                        monitor.height().unwrap_or(1080) as f32,
                    ),
                    transparent: true,
                    decorations: false,
                    ..Default::default()
                });

                let capture_view = CaptureView::build_with_capture_buffer(capture_buffer);
                self.windows
                    .insert(id, AppWindow::CaptureOverlay(capture_view));
                self.status = global_constants::STATUS_OVERLAY_READY.to_string();
                self.log_info_event(
                    "capture_overlay_created",
                    serde_json::json!({"window_id": format!("{:?}", id)}),
                );

                return task.discard().chain(window::gain_focus(id));
            }
            Err(monitor_error) => {
                self.log_error_event(
                    "capture_overlay_monitor_failed",
                    serde_json::json!({"error": monitor_error.to_string()}),
                );
                self.status = format!(
                    "{}{}",
                    global_constants::CAPTURE_ERROR_MONITOR_PREFIX,
                    monitor_error
                );
            }
        }
        Task::none()
    }

    pub(super) fn handle_capture_error(&mut self, error_msg: String) -> Task<OrchestratorMessage> {
        self.log_error_event(
            "capture_error_received",
            serde_json::json!({"error": error_msg}),
        );

        let user_friendly_message = build_capture_error_message(&error_msg);
        self.status = user_friendly_message;

        Task::none()
    }

    pub(super) fn handle_escape_pressed(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("escape_pressed", serde_json::json!({}));
        if let Some((id, AppWindow::CaptureOverlay(_))) = self
            .windows
            .iter()
            .find(|(_, w)| matches!(w, AppWindow::CaptureOverlay(_)))
        {
            self.log_info_event(
                "capture_overlay_closing_on_escape",
                serde_json::json!({"window_id": format!("{:?}", id)}),
            );
            return window::close(*id);
        }
        self.log_info_event("escape_pressed_no_overlay", serde_json::json!({}));
        self.status = global_constants::STATUS_READY.to_string();
        Task::none()
    }

    pub(super) fn handle_capture_overlay_message(
        &mut self,
        window_id: Id,
        capture_msg: CaptureViewMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "capture_overlay_message_received",
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "message": format!("{:?}", capture_msg),
            }),
        );
        if let CaptureViewMessage::ConfirmSelection = capture_msg {
            return self.update(OrchestratorMessage::ConfirmSelection(window_id));
        }

        if let CaptureViewMessage::SelectWindow = capture_msg {
            self.log_info_event(
                "window_selection_requested_from_overlay",
                serde_json::json!({}),
            );
            return Task::batch(vec![
                window::close(window_id),
                Task::done(OrchestratorMessage::OpenWindowPicker),
            ]);
        }

        if let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get_mut(&window_id) {
            capture_view.update(capture_msg);
        } else {
            self.log_error_event(
                "capture_overlay_window_not_found",
                serde_json::json!({"window_id": format!("{:?}", window_id)}),
            );
        }
        Task::none()
    }

    pub(super) fn handle_confirm_selection(&mut self, overlay_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "selection_confirming",
            serde_json::json!({"overlay_id": format!("{:?}", overlay_id)}),
        );

        let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get(&overlay_id) else {
            self.log_error_event("confirm_selection_overlay_not_found", serde_json::json!({}));
            return window::close(overlay_id);
        };

        let Some(selection_rect) = capture_view.get_selected_region() else {
            self.log_error_event("confirm_selection_no_region", serde_json::json!({}));
            return window::close(overlay_id);
        };

        self.log_info_event(
            "selection_confirmed",
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

    pub(super) fn handle_show_cropped_image(
        &mut self,
        capture_buffer: CaptureBuffer,
        selection_rect: Rectangle,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "cropped_image_showing",
            serde_json::json!({"rect": format!("{:?}", selection_rect)}),
        );

        let cropped_buffer = capture_buffer.crop_region(
            selection_rect.x as u32,
            selection_rect.y as u32,
            selection_rect.width as u32,
            selection_rect.height as u32,
        );

        match cropped_buffer {
            Ok(buffer) => {
                self.log_info_event(
                    "image_cropped",
                    serde_json::json!({"width": buffer.width, "height": buffer.height}),
                );

                let (id, task) = window::open(window::Settings {
                    size: Size::new(
                        (buffer.width as f32).min(1200.0),
                        (buffer.height as f32).min(800.0),
                    ),
                    position: window::Position::Centered,
                    resizable: true,
                    ..Default::default()
                });

                let mut view = crate::presentation::InteractiveOcrView::build(
                    buffer.clone(),
                    self.settings.theme_mode.clone(),
                );

                if let Some(strokes) = self.pending_draw_strokes.take() {
                    view.set_draw_strokes(strokes);
                }

                self.windows.insert(id, AppWindow::InteractiveOcr(view));
                self.status = global_constants::STATUS_READY_SIMPLE.to_string();

                return task.discard();
            }
            Err(crop_error) => {
                self.log_error_event(
                    "image_crop_failed",
                    serde_json::json!({"error": crop_error.to_string()}),
                );
                self.status = format!(
                    "{}{}",
                    global_constants::CAPTURE_ERROR_CROP_PREFIX,
                    crop_error
                );
            }
        }
        Task::none()
    }
}

pub(super) fn build_capture_error_message(error_msg: &str) -> String {
    #[cfg(target_os = "linux")]
    let platform = global_constants::CAPTURE_PLATFORM_LINUX;
    #[cfg(target_os = "macos")]
    let platform = global_constants::CAPTURE_PLATFORM_MACOS;
    #[cfg(target_os = "windows")]
    let platform = global_constants::CAPTURE_PLATFORM_WINDOWS;
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let platform = global_constants::CAPTURE_PLATFORM_UNKNOWN;

    build_capture_error_message_for_platform(error_msg, platform)
}

pub(super) fn build_capture_error_message_for_platform(error_msg: &str, platform: &str) -> String {
    let error_lower = error_msg.to_lowercase();

    match platform {
        global_constants::CAPTURE_PLATFORM_LINUX => {
            let is_permission_error = error_lower
                .contains(global_constants::CAPTURE_ERROR_KEYWORD_PERMISSION)
                || error_lower.contains(global_constants::CAPTURE_ERROR_KEYWORD_ACCESS)
                || error_lower.contains(global_constants::CAPTURE_ERROR_KEYWORD_DENIED)
                || error_lower.contains(global_constants::CAPTURE_ERROR_KEYWORD_PIPEWIRE)
                || error_lower.contains(global_constants::CAPTURE_ERROR_KEYWORD_PORTAL);

            if is_permission_error {
                return format!(
                    "{}{}{}",
                    global_constants::CAPTURE_ERROR_LINUX_PERMISSION_PREFIX,
                    error_msg,
                    global_constants::CAPTURE_ERROR_LINUX_PERMISSION_SUFFIX
                );
            }
        }
        global_constants::CAPTURE_PLATFORM_MACOS => {
            let is_permission_error = error_lower
                .contains(global_constants::CAPTURE_ERROR_KEYWORD_PERMISSION)
                || error_lower.contains(global_constants::CAPTURE_ERROR_KEYWORD_ACCESS)
                || error_lower.contains(global_constants::CAPTURE_ERROR_KEYWORD_DENIED);

            if is_permission_error {
                return format!(
                    "{}{}{}",
                    global_constants::CAPTURE_ERROR_MACOS_PERMISSION_PREFIX,
                    error_msg,
                    global_constants::CAPTURE_ERROR_MACOS_PERMISSION_SUFFIX
                );
            }
        }
        _ => {}
    }

    format!(
        "{}{}{}",
        global_constants::CAPTURE_ERROR_GENERIC_PREFIX,
        error_msg,
        global_constants::CAPTURE_ERROR_GENERIC_SUFFIX
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_capture_error_message_linux_permission_error() {
        let error = "Access denied to screen capture";
        let result = build_capture_error_message_for_platform(error, "linux");
        assert!(result.contains("Screen capture failed"));
        assert!(result.contains("PipeWire"));
        assert!(result.contains("xdg-desktop-portal"));
        assert!(result.contains("sudo apt install"));
    }

    #[test]
    fn test_build_capture_error_message_linux_pipewire_error() {
        let error = "PipeWire connection failed";
        let result = build_capture_error_message_for_platform(error, "linux");
        assert!(result.contains("Screen capture failed"));
        assert!(result.contains("PipeWire"));
    }

    #[test]
    fn test_build_capture_error_message_linux_portal_error() {
        let error = "Portal request denied";
        let result = build_capture_error_message_for_platform(error, "linux");
        assert!(result.contains("Screen capture failed"));
        assert!(result.contains("xdg-desktop-portal"));
    }

    #[test]
    fn test_build_capture_error_message_linux_generic_error() {
        let error = "Unknown capture error";
        let result = build_capture_error_message_for_platform(error, "linux");
        assert!(result.contains("Capture failed"));
        assert!(result.contains("Try closing other instances"));
        assert!(!result.contains("PipeWire"));
    }

    #[test]
    fn test_build_capture_error_message_macos_permission_error() {
        let error = "Permission denied for screen recording";
        let result = build_capture_error_message_for_platform(error, "macos");
        assert!(result.contains("Screen capture failed"));
        assert!(result.contains("Screen Recording permission"));
        assert!(result.contains("System Settings"));
    }

    #[test]
    fn test_build_capture_error_message_macos_access_error() {
        let error = "Access to screen not granted";
        let result = build_capture_error_message_for_platform(error, "macos");
        assert!(result.contains("Screen capture failed"));
        assert!(result.contains("Privacy & Security"));
    }

    #[test]
    fn test_build_capture_error_message_macos_generic_error() {
        let error = "Monitor not found";
        let result = build_capture_error_message_for_platform(error, "macos");
        assert!(result.contains("Capture failed"));
        assert!(result.contains("Try closing other instances"));
        assert!(!result.contains("System Settings"));
    }

    #[test]
    fn test_build_capture_error_message_windows_always_generic() {
        let error = "Permission denied";
        let result = build_capture_error_message_for_platform(error, "windows");
        assert!(result.contains("Capture failed"));
        assert!(result.contains("Try closing other instances"));
        assert!(!result.contains("PipeWire"));
        assert!(!result.contains("System Settings"));
    }

    #[test]
    fn test_build_capture_error_message_unknown_platform() {
        let error = "Some error";
        let result = build_capture_error_message_for_platform(error, "freebsd");
        assert!(result.contains("Capture failed"));
        assert!(result.contains("Try closing other instances"));
    }
}
