use super::*;
use crate::adapters::{auto_launch, macos_permissions};

impl AppOrchestrator {
    pub(super) fn handle_open_onboarding(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("onboarding_window_opening", serde_json::json!({}));

        let existing_id = self.onboarding_window_id;

        if existing_id.is_some_and(|window_id| self.windows.contains_key(&window_id)) {
            self.log_warn_event("onboarding_window_already_open", serde_json::json!({}));
            return Task::none();
        }

        let onboarding_view = OnboardingView::new(
            macos_permissions::macos::check_screen_recording_permission(),
            macos_permissions::macos::check_input_monitoring_permission(),
            auto_launch::is_launch_at_login_enabled(),
        );

        let (id, task) = window::open(window::Settings {
            size: Size::new(600.0, 800.0),
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });

        self.onboarding_window_id = Some(id);
        self.windows
            .insert(id, AppWindow::Onboarding(onboarding_view));
        self.log_info_event(
            "onboarding_window_created",
            serde_json::json!({"window_id": format!("{:?}", id)}),
        );

        task.discard()
    }

    pub(super) fn handle_onboarding_message(
        &mut self,
        window_id: Id,
        message: OnboardingMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_debug_event(
            "onboarding_message_received",
            serde_json::json!({"message": format!("{:?}", message)}),
        );

        match message {
            OnboardingMessage::OpenScreenRecordingSettings => {
                if !macos_permissions::macos::open_screen_recording_settings() {
                    self.log_error_event(
                        "onboarding_screen_recording_settings_failed",
                        serde_json::json!({}),
                    );
                    self.status =
                        global_constants::STATUS_ONBOARDING_SCREEN_RECORDING_FAILED.to_string();
                }
                return Task::none();
            }
            OnboardingMessage::OpenInputMonitoringSettings => {
                if !macos_permissions::macos::open_input_monitoring_settings() {
                    self.log_error_event(
                        "onboarding_input_monitoring_settings_failed",
                        serde_json::json!({}),
                    );
                    self.status =
                        global_constants::STATUS_ONBOARDING_INPUT_MONITORING_FAILED.to_string();
                }
                return Task::none();
            }
            OnboardingMessage::RefreshPermissions => {
                return self.refresh_onboarding_permissions(window_id);
            }
            OnboardingMessage::FinishOnboarding => {
                return self.handle_finish_onboarding(window_id);
            }
            _ => {}
        }

        if let Some(AppWindow::Onboarding(view)) = self.windows.get_mut(&window_id) {
            view.handle_message(message);
        }

        Task::none()
    }

    fn refresh_onboarding_permissions(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let screen_recording_granted =
            macos_permissions::macos::check_screen_recording_permission();
        let input_monitoring_granted =
            macos_permissions::macos::check_input_monitoring_permission();

        self.log_info_event(
            "onboarding_permissions_refreshed",
            serde_json::json!({
                "screen_recording": screen_recording_granted,
                "input_monitoring": input_monitoring_granted,
            }),
        );

        if let Some(AppWindow::Onboarding(view)) = self.windows.get_mut(&window_id) {
            view.update_permissions(screen_recording_granted, input_monitoring_granted);
        }

        Task::none()
    }

    fn handle_finish_onboarding(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event("onboarding_finishing", serde_json::json!({}));

        let launch_at_login = match self.windows.get(&window_id) {
            Some(AppWindow::Onboarding(view)) => view.is_launch_at_login_enabled(),
            _ => false,
        };

        self.settings.onboarding_complete = true;
        self.settings.launch_at_login = launch_at_login;

        if let Err(save_error) = self.settings.save() {
            self.log_error_event(
                "onboarding_settings_save_failed",
                serde_json::json!({"error": save_error.to_string()}),
            );
        }

        auto_launch::set_launch_at_login(launch_at_login);
        self.log_info_event(
            "onboarding_finished",
            serde_json::json!({"launch_at_login": launch_at_login}),
        );

        self.windows.remove(&window_id);
        self.onboarding_window_id = None;

        Task::batch(vec![
            window::close(window_id),
            Task::done(OrchestratorMessage::OpenMainWindow),
            Task::done(OrchestratorMessage::EnableKeyboardListener),
        ])
    }
}
