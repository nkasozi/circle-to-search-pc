use iced::window::Id;
use iced::{Element, Subscription, Task};

use crate::adapters::macos_app_behavior;
use crate::core::models::HotkeyRegistration;
use crate::core::orchestrators::app_composition::{
    apply_hotkey_bindings, build_hotkey_bindings, build_hotkey_listener,
};
use crate::core::orchestrators::app_orchestrator::{AppOrchestrator, OrchestratorMessage};
use crate::ports::{GlobalHotkeyListener, SystemTray};

mod constants;
mod permissions;
mod placeholder_ocr;
mod startup;
mod subscription_sources;

use constants::{EVENT_APP_INITIALIZING, LOG_TAG};

pub struct CircleApp {
    orchestrator: AppOrchestrator,
    tray: Option<SystemTray>,
    hotkey_listener: Option<GlobalHotkeyListener>,
    hotkey_registrations: Vec<HotkeyRegistration>,
    tray_shows_recording: bool,
}

impl CircleApp {
    pub fn build() -> (Self, Task<OrchestratorMessage>) {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_APP_INITIALIZING})
        );
        let _ = macos_app_behavior::macos::hide_dock_icon();
        let settings = startup::load_settings();
        let onboarding_complete = settings.onboarding_complete;
        let orchestrator = startup::build_orchestrator(settings);
        let tray = startup::build_tray(&orchestrator);
        let all_permissions_granted = permissions::check_all_permissions_granted();
        let needs_onboarding = !onboarding_complete || !all_permissions_granted;
        let hotkey_bindings = build_hotkey_bindings(orchestrator.current_settings());
        let mut hotkey_listener = build_hotkey_listener();
        let hotkey_outcome = apply_hotkey_bindings(hotkey_listener.as_mut(), &hotkey_bindings);
        let launch_message = startup::choose_launch_message(
            needs_onboarding,
            orchestrator.current_settings().show_home_on_launch,
        );
        let tasks = startup::build_startup_tasks(hotkey_outcome.failures.clone(), launch_message);
        (
            Self {
                orchestrator,
                tray,
                hotkey_listener,
                hotkey_registrations: hotkey_outcome.registrations,
                tray_shows_recording: false,
            },
            Task::batch(tasks),
        )
    }

    pub fn handle_update(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        if matches!(message, OrchestratorMessage::ReapplyHotkeys) {
            return self.reapply_hotkeys();
        }
        let task = self.orchestrator.update(message);
        let _ = self.sync_tray_recording_state();
        task
    }

    fn sync_tray_recording_state(&mut self) -> bool {
        let is_recording = self.orchestrator.is_recording();
        if is_recording == self.tray_shows_recording {
            return false;
        }
        self.tray_shows_recording = is_recording;
        self.tray
            .as_ref()
            .map(|tray| tray.show_recording_state(is_recording))
            .is_some()
    }

    pub fn render_view(&self, window_id: Id) -> Element<'_, OrchestratorMessage> {
        self.orchestrator.render_view(window_id)
    }

    pub fn window_title(&self, window_id: Id) -> String {
        self.orchestrator.window_title(window_id)
    }

    pub fn window_theme(&self, window_id: Id) -> iced::Theme {
        self.orchestrator.window_theme(window_id)
    }

    pub fn handle_subscription(&self) -> Subscription<OrchestratorMessage> {
        let mut subscriptions = vec![
            subscription_sources::window_event_subscription(),
            subscription_sources::tray_poll_subscription(),
            subscription_sources::spinner_tick_subscription(),
            self.orchestrator.recording_subscription(),
            self.orchestrator.player_subscription(),
        ];
        subscriptions.extend(subscription_sources::onboarding_poll_subscription(
            self.orchestrator.onboarding_permission_poll_needed(),
        ));
        subscriptions.extend(subscription_sources::global_hotkey_subscription(
            &self.hotkey_registrations,
        ));
        Subscription::batch(subscriptions)
    }

    fn reapply_hotkeys(&mut self) -> Task<OrchestratorMessage> {
        let hotkey_bindings = build_hotkey_bindings(self.orchestrator.current_settings());
        let outcome = apply_hotkey_bindings(self.hotkey_listener.as_mut(), &hotkey_bindings);
        self.hotkey_registrations = outcome.registrations;
        Task::done(OrchestratorMessage::HotkeysApplied(outcome.failures))
    }
}
