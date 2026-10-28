use super::*;
use crate::adapters::{auto_launch, macos_permissions};
use crate::presentation::OnboardingSetup;

const ONBOARDING_WINDOW_SIZE: Size = Size::new(600.0, 680.0);
const EVENT_ONBOARDING_WINDOW_OPENING: &str = "onboarding_window_opening";
const EVENT_ONBOARDING_MESSAGE_RECEIVED: &str = "onboarding_message_received";
const EVENT_ONBOARDING_SETTINGS_OPENED: &str = "onboarding_settings_opened";
const EVENT_ONBOARDING_SCREEN_ACCESS_REQUESTED: &str = "onboarding_screen_access_requested";
const EVENT_ONBOARDING_SCREEN_ACCESS_CHANGED: &str = "onboarding_screen_access_changed";
const EVENT_ONBOARDING_SETTINGS_SAVE_FAILED: &str = "onboarding_settings_save_failed";
const EVENT_ONBOARDING_FINISHED: &str = "onboarding_finished";
const DEFAULT_LAUNCH_AT_LOGIN_WITHOUT_VIEW: bool = false;
const DEFAULT_SHOW_HOME_ON_LAUNCH_WITHOUT_VIEW: bool = true;

impl AppOrchestrator {
    pub(super) fn handle_open_onboarding(&mut self) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(EVENT_ONBOARDING_WINDOW_OPENING, serde_json::json!({}));
        if self
            .onboarding_window_id
            .is_some_and(|window_id| self.windows.contains_key(&window_id))
        {
            return Task::none();
        }
        let onboarding_view = OnboardingView::new(OnboardingSetup {
            screen_access_granted: macos_permissions::macos::check_screen_recording_permission(),
            launch_at_login: auto_launch::is_launch_at_login_enabled(),
            show_home_on_launch: self.settings.show_home_on_launch,
            capture_hotkey: self.settings.capture_hotkey.clone(),
            recording_hotkey: self.settings.recording_hotkey.clone(),
        });
        let (onboarding_window_id, open_task) = window::open(window::Settings {
            size: ONBOARDING_WINDOW_SIZE,
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });
        self.onboarding_window_id = Some(onboarding_window_id);
        self.windows
            .insert(onboarding_window_id, AppWindow::Onboarding(onboarding_view));
        crate::adapters::macos_app_behavior::macos::activate_app_ignoring_other_apps();
        open_task
            .discard()
            .chain(window::gain_focus(onboarding_window_id))
    }

    pub(super) fn handle_onboarding_message(
        &mut self,
        window_id: Id,
        message: OnboardingMessage,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_debug_event(
            EVENT_ONBOARDING_MESSAGE_RECEIVED,
            serde_json::json!({"message": format!("{:?}", message)}),
        );
        let is_finished = match self.windows.get_mut(&window_id) {
            Some(AppWindow::Onboarding(view)) => view.handle_message(&message),
            _ => false,
        };
        match message {
            OnboardingMessage::RequestScreenAccess => self.request_screen_access(window_id),
            OnboardingMessage::OpenScreenRecordingSettings => {
                let opened = macos_permissions::macos::open_screen_recording_settings();
                let _ = self.log_info_event(
                    EVENT_ONBOARDING_SETTINGS_OPENED,
                    serde_json::json!({"opened": opened}),
                );
                Task::none()
            }
            OnboardingMessage::RefreshPermissions => self.refresh_onboarding_permissions(window_id),
            OnboardingMessage::RestartApp => self.handle_restart_app(),
            _ if is_finished => self.handle_finish_onboarding(window_id),
            _ => Task::none(),
        }
    }

    fn request_screen_access(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let granted = macos_permissions::macos::request_screen_recording_permission();
        let _ = self.log_info_event(
            EVENT_ONBOARDING_SCREEN_ACCESS_REQUESTED,
            serde_json::json!({"granted": granted}),
        );
        if !granted {
            let _ = macos_permissions::macos::open_screen_recording_settings();
        }
        self.refresh_onboarding_permissions(window_id)
    }

    fn refresh_onboarding_permissions(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let screen_access_granted = macos_permissions::macos::check_screen_recording_permission();
        let access_changed = match self.windows.get_mut(&window_id) {
            Some(AppWindow::Onboarding(view)) => view.update_screen_access(screen_access_granted),
            _ => false,
        };
        if access_changed {
            let _ = self.log_info_event(
                EVENT_ONBOARDING_SCREEN_ACCESS_CHANGED,
                serde_json::json!({"granted": screen_access_granted}),
            );
        }
        Task::none()
    }

    pub fn onboarding_permission_poll_needed(&self) -> Option<Id> {
        let window_id = self.onboarding_window_id?;
        match self.windows.get(&window_id) {
            Some(AppWindow::Onboarding(view)) if view.is_on_screen_access_step() => Some(window_id),
            _ => None,
        }
    }

    fn handle_finish_onboarding(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let (launch_at_login, show_home_on_launch) = match self.windows.get(&window_id) {
            Some(AppWindow::Onboarding(view)) => (
                view.is_launch_at_login_enabled(),
                view.is_show_home_on_launch_enabled(),
            ),
            _ => (
                DEFAULT_LAUNCH_AT_LOGIN_WITHOUT_VIEW,
                DEFAULT_SHOW_HOME_ON_LAUNCH_WITHOUT_VIEW,
            ),
        };
        self.settings.onboarding_complete = true;
        self.settings.launch_at_login = launch_at_login;
        self.settings.show_home_on_launch = show_home_on_launch;
        if let Err(save_error) = self.settings.save() {
            let _ = self.log_error_event(
                EVENT_ONBOARDING_SETTINGS_SAVE_FAILED,
                serde_json::json!({"error": save_error.to_string()}),
            );
        }
        let launch_applied = auto_launch::set_launch_at_login(launch_at_login);
        let _ = self.log_info_event(EVENT_ONBOARDING_FINISHED, serde_json::json!({"launch_at_login": launch_at_login, "launch_applied": launch_applied}));
        self.windows.remove(&window_id);
        self.onboarding_window_id = None;
        Task::batch(vec![
            window::close(window_id),
            Task::done(OrchestratorMessage::OpenMainWindow),
        ])
    }
}
