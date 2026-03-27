use super::*;

impl AppOrchestrator {
    pub(super) fn handle_open_window_picker(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("window_picker_open_requested", serde_json::json!({}));

        if let Some(id) = self.window_picker_window_id {
            log::debug!("[ORCHESTRATOR] Window picker already open, bringing to front");
            return window::gain_focus(id);
        }

        let mut picker_view = WindowPickerView::build(vec![]);
        picker_view.set_loading(true);

        let (id, open_task) = window::open(window::Settings {
            size: Size::new(500.0, 600.0),
            position: window::Position::Centered,
            visible: true,
            resizable: true,
            decorations: true,
            ..Default::default()
        });

        self.window_picker_window_id = Some(id);
        self.windows
            .insert(id, AppWindow::WindowPicker(picker_view));

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            open_task.discard(),
            Task::future(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                match screen_capturer.list_capturable_windows() {
                    Ok(windows) => {
                        AppOrchestrator::log_info_event_for_correlation(
                            correlation_id.clone(),
                            "window_picker_windows_loaded",
                            serde_json::json!({
                                "window_count": windows.len(),
                            }),
                        );
                        OrchestratorMessage::WindowsListLoaded(id, windows)
                    }
                    Err(error) => {
                        AppOrchestrator::log_error_event_for_correlation(
                            correlation_id.clone(),
                            "window_picker_windows_load_failed",
                            serde_json::json!({
                                "error": error.to_string(),
                            }),
                        );
                        OrchestratorMessage::WindowsListLoaded(id, vec![])
                    }
                }
            }),
        ])
    }

    pub(super) fn handle_window_picker_message(
        &mut self,
        window_id: Id,
        msg: WindowPickerMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "window_picker_message_received",
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "message": format!("{:?}", msg),
            }),
        );

        match msg {
            WindowPickerMessage::WindowSelected(selected_id) => {
                if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
                    view.update(WindowPickerMessage::WindowSelected(selected_id));
                }
                Task::none()
            }
            WindowPickerMessage::ConfirmSelection => {
                self.confirm_window_picker_selection(window_id)
            }
            WindowPickerMessage::Cancel => {
                self.log_info_event(
                    "window_picker_cancelled",
                    serde_json::json!({
                        "window_id": format!("{:?}", window_id),
                    }),
                );
                self.window_picker_window_id = None;
                window::close(window_id)
            }
            WindowPickerMessage::RefreshWindows => self.refresh_window_picker_windows(window_id),
            WindowPickerMessage::CaptureFullScreen => {
                self.log_info_event(
                    "window_picker_full_screen_selected",
                    serde_json::json!({
                        "window_id": format!("{:?}", window_id),
                    }),
                );
                Task::batch(vec![
                    window::close(window_id),
                    Task::done(OrchestratorMessage::CaptureScreen),
                ])
            }
            WindowPickerMessage::SpinnerTick => Task::none(),
            WindowPickerMessage::FilterChanged(query) => {
                if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
                    view.update(WindowPickerMessage::FilterChanged(query));
                }
                Task::none()
            }
        }
    }

    fn confirm_window_picker_selection(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let selected_app_name =
            if let Some(AppWindow::WindowPicker(view)) = self.windows.get(&window_id) {
                view.get_selected_window_info()
                    .map(|window_info| window_info.app_name.clone())
            } else {
                None
            };

        let Some(app_name) = selected_app_name else {
            return Task::none();
        };

        self.window_picker_window_id = None;
        Task::batch(vec![
            window::close(window_id),
            Task::done(OrchestratorMessage::FocusWindowAndCapture(app_name)),
        ])
    }

    fn refresh_window_picker_windows(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
            view.set_loading(true);
        }

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();
        Task::future(async move {
            match screen_capturer.list_capturable_windows() {
                Ok(windows) => {
                    AppOrchestrator::log_info_event_for_correlation(
                        correlation_id.clone(),
                        "window_picker_windows_refreshed",
                        serde_json::json!({
                            "window_count": windows.len(),
                        }),
                    );
                    OrchestratorMessage::WindowsListLoaded(window_id, windows)
                }
                Err(error) => {
                    AppOrchestrator::log_error_event_for_correlation(
                        correlation_id.clone(),
                        "window_picker_windows_refresh_failed",
                        serde_json::json!({
                            "error": error.to_string(),
                        }),
                    );
                    OrchestratorMessage::WindowsListLoaded(window_id, vec![])
                }
            }
        })
    }

    pub(super) fn handle_windows_list_loaded(
        &mut self,
        window_id: Id,
        windows: Vec<WindowInfo>,
    ) -> Task<OrchestratorMessage> {
        log::debug!(
            "[ORCHESTRATOR] Windows list loaded: {} windows",
            windows.len()
        );

        if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
            view.set_windows(windows);
        }

        Task::none()
    }

    pub(super) fn handle_focus_window_and_capture(
        &mut self,
        app_name: String,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "focus_window_and_capture_started",
            serde_json::json!({
                "app_name": app_name,
            }),
        );

        let correlation_id = self.current_correlation_id();
        let app_name_for_task = app_name.clone();

        Task::future(async move {
            let _ =
                crate::infrastructure::utils::focus_external_window_by_app_name(&app_name_for_task);
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id,
                "focus_window_and_capture_completed",
                serde_json::json!({}),
            );
            OrchestratorMessage::CaptureScreen
        })
    }

    pub(super) fn handle_capture_selected_window(
        &mut self,
        window_id: u32,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "window_capture_selected_started",
            serde_json::json!({
                "selected_window_id": window_id,
            }),
        );
        self.window_picker_window_id = None;

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            match screen_capturer.capture_window_by_id(window_id) {
                Ok(capture_buffer) => {
                    AppOrchestrator::log_info_event_for_correlation(
                        correlation_id.clone(),
                        "window_capture_selected_completed",
                        serde_json::json!({
                            "selected_window_id": window_id,
                            "width": capture_buffer.width,
                            "height": capture_buffer.height,
                        }),
                    );
                    OrchestratorMessage::WindowCaptureComplete(capture_buffer)
                }
                Err(error) => {
                    AppOrchestrator::log_error_event_for_correlation(
                        correlation_id,
                        "window_capture_selected_failed",
                        serde_json::json!({
                            "selected_window_id": window_id,
                            "error": error.to_string(),
                        }),
                    );
                    OrchestratorMessage::WindowCaptureError(error.to_string())
                }
            }
        })
    }

    pub(super) fn handle_window_capture_complete(
        &mut self,
        capture_buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        log::info!(
            "[ORCHESTRATOR] Processing window capture: {}x{}",
            capture_buffer.width,
            capture_buffer.height
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
