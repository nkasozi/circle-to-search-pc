use super::*;
use crate::adapters::macos_app_behavior;

impl AppOrchestrator {
    pub(super) fn handle_open_main_window(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("main_window_opening", serde_json::json!({}));

        let ocr_window_ids: Vec<Id> = self
            .windows
            .iter()
            .filter_map(|(id, window)| {
                matches!(window, AppWindow::InteractiveOcr(_)).then_some(*id)
            })
            .collect();

        let focus_ocr_windows: Vec<Task<OrchestratorMessage>> = ocr_window_ids
            .iter()
            .map(|id| {
                self.log_info_event(
                    "capture_result_window_bringing_to_front",
                    serde_json::json!({"window_id": format!("{:?}", id)}),
                );
                window::minimize(*id, false).chain(window::gain_focus(*id))
            })
            .collect();

        macos_app_behavior::macos::activate_app_ignoring_other_apps();

        if let Some(id) = self
            .main_window_id
            .filter(|id| self.windows.contains_key(id))
        {
            self.log_info_event("main_window_already_exists", serde_json::json!({}));
            return Task::batch(
                std::iter::once(window::gain_focus(id))
                    .chain(focus_ocr_windows)
                    .collect::<Vec<_>>(),
            );
        }

        let (id, task) = window::open(window::Settings {
            size: Size::new(700.0, 800.0),
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });

        self.main_window_id = Some(id);
        self.windows.insert(id, AppWindow::Main);
        self.log_info_event(
            "main_window_created",
            serde_json::json!({"window_id": format!("{:?}", id)}),
        );
        Task::batch(
            std::iter::once(task.discard().chain(window::gain_focus(id)))
                .chain(focus_ocr_windows)
                .collect::<Vec<_>>(),
        )
    }

    pub(super) fn handle_window_closed(&mut self, id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "window_closed",
            serde_json::json!({"window_id": format!("{:?}", id)}),
        );

        if Some(id) == self.hidden_window_id {
            self.log_error_event(
                "hidden_window_closed_unexpectedly",
                serde_json::json!({"window_id": format!("{:?}", id)}),
            );
            self.hidden_window_id = None;
            self.windows.remove(&id);
            return self.create_hidden_window();
        }

        if Some(id) == self.main_window_id {
            self.log_info_event("main_window_closed", serde_json::json!({}));
            self.windows.remove(&id);
            self.main_window_id = None;
            return Task::none();
        }

        if Some(id) == self.onboarding_window_id {
            self.log_info_event("onboarding_window_closed", serde_json::json!({}));
            self.windows.remove(&id);
            self.onboarding_window_id = None;
            return Task::none();
        }

        if Some(id) == self.window_picker_window_id {
            self.log_info_event("window_picker_closed", serde_json::json!({}));
            self.windows.remove(&id);
            self.window_picker_window_id = None;
            return Task::none();
        }

        let was_ocr_window = matches!(self.windows.get(&id), Some(AppWindow::InteractiveOcr(_)));
        self.windows.remove(&id);
        if Some(id) == self.settings_window_id {
            self.settings_window_id = None;
            self.discard_settings_edit();
        }
        self.log_info_event(
            "window_removed_from_tracking",
            serde_json::json!({"remaining_windows": self.windows.len()}),
        );
        self.status = global_constants::STATUS_READY.to_string();

        if !was_ocr_window {
            return Task::none();
        }

        let Some(main_id) = self.main_window_id else {
            return Task::none();
        };

        window::minimize(main_id, false)
    }

    pub(super) fn handle_restart_app(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("app_restart_requested", serde_json::json!({}));
        let exe_path = match std::env::current_exe() {
            Ok(path) => path,
            Err(path_error) => {
                self.log_error_event(
                    "app_restart_exe_path_failed",
                    serde_json::json!({"error": path_error.to_string()}),
                );
                return Task::none();
            }
        };

        match std::process::Command::new(exe_path).spawn() {
            Ok(_) => {
                self.log_info_event("app_restart_new_process_launched", serde_json::json!({}));
                std::process::exit(0);
            }
            Err(spawn_error) => {
                self.log_error_event(
                    "app_restart_spawn_failed",
                    serde_json::json!({"error": spawn_error.to_string()}),
                );
                Task::none()
            }
        }
    }

    pub(super) fn handle_window_focused(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let is_hidden_window = self.hidden_window_id == Some(window_id);
        let is_main_window = self.main_window_id == Some(window_id);

        if !is_hidden_window && !is_main_window {
            return Task::none();
        }

        let ocr_window_ids: Vec<Id> = self
            .windows
            .iter()
            .filter_map(|(id, window)| {
                matches!(window, AppWindow::InteractiveOcr(_)).then_some(*id)
            })
            .collect();

        if ocr_window_ids.is_empty() {
            return Task::none();
        }

        self.log_info_event(
            "app_window_focused_recovering_capture_result_windows",
            serde_json::json!({"ocr_window_count": ocr_window_ids.len()}),
        );

        let focus_tasks: Vec<Task<OrchestratorMessage>> = ocr_window_ids
            .iter()
            .map(|id| window::minimize(*id, false).chain(window::gain_focus(*id)))
            .collect();

        macos_app_behavior::macos::activate_app_ignoring_other_apps();

        Task::batch(focus_tasks)
    }

    pub(super) fn handle_tray_event(&mut self, event: TrayEvent) -> Task<OrchestratorMessage> {
        self.log_info_event(
            "tray_event_received",
            serde_json::json!({"event": format!("{:?}", event)}),
        );

        match event {
            TrayEvent::ShowWindow => self.handle_open_main_window(),
            TrayEvent::SelectWindow => self.handle_open_window_picker(),
            TrayEvent::OpenSettings => self.handle_open_settings(),
            TrayEvent::Quit => {
                self.log_info_event("quit_requested_from_tray", serde_json::json!({}));
                iced::exit()
            }
        }
    }

    pub(super) fn handle_hide_main_window(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("main_window_hiding", serde_json::json!({}));

        if let Some(id) = self.main_window_id {
            return window::close(id);
        }

        Task::none()
    }
}
