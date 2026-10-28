use super::*;
use crate::presentation::settings_view::{
    apply_settings_change, build_accelerator, PressedModifiers, SettingsChange, SettingsMessage,
    SettingsViewState,
};

mod saving;
mod validation;

const SETTINGS_WINDOW_SIZE: Size = Size::new(680.0, 760.0);
const SETTINGS_WINDOW_MINIMUM_SIZE: Size = Size::new(560.0, 560.0);
const EVENT_SETTINGS_WINDOW_OPENING: &str = "settings_window_opening";
const EVENT_SETTINGS_DRAFT_CHANGED: &str = "settings_draft_changed";

impl AppOrchestrator {
    pub(super) fn handle_open_settings(&mut self) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(EVENT_SETTINGS_WINDOW_OPENING, serde_json::json!({}));
        if let Some(settings_window_id) = self.settings_window_id {
            return window::gain_focus(settings_window_id);
        }
        let (settings_window_id, open_task) = window::open(window::Settings {
            size: SETTINGS_WINDOW_SIZE,
            min_size: Some(SETTINGS_WINDOW_MINIMUM_SIZE),
            position: window::Position::Centered,
            ..Default::default()
        });
        self.settings_window_id = Some(settings_window_id);
        let _ = self.begin_settings_edit();
        self.windows.insert(
            settings_window_id,
            AppWindow::Settings(SettingsViewState::default()),
        );
        crate::adapters::macos_app_behavior::macos::activate_app_ignoring_other_apps();
        open_task
            .discard()
            .chain(window::gain_focus(settings_window_id))
    }

    fn settings_view_state_mut(&mut self) -> Option<&mut SettingsViewState> {
        let window_id = self.settings_window_id?;
        match self.windows.get_mut(&window_id) {
            Some(AppWindow::Settings(state)) => Some(state),
            _ => None,
        }
    }

    pub(super) fn handle_settings_message(
        &mut self,
        message: SettingsMessage,
    ) -> Task<OrchestratorMessage> {
        match message {
            SettingsMessage::SelectTab(tab) => self
                .settings_view_state_mut()
                .map(|state| state.select_tab(tab))
                .map(|_| Task::none())
                .unwrap_or_else(Task::none),
            SettingsMessage::Change(change) => self.apply_draft_change(change),
            SettingsMessage::StartShortcutCapture(action) => self
                .settings_view_state_mut()
                .map(|state| state.begin_shortcut_capture(action))
                .map(|_| Task::none())
                .unwrap_or_else(Task::none),
            SettingsMessage::CancelShortcutCapture => self
                .settings_view_state_mut()
                .map(|state| state.finish_shortcut_capture())
                .map(|_| Task::none())
                .unwrap_or_else(Task::none),
            SettingsMessage::ChooseRecordingsFolder => self.choose_recordings_folder(),
            SettingsMessage::RecordingsFolderChosen(Some(folder)) => {
                self.apply_draft_change(SettingsChange::RecordingsFolder(folder))
            }
            SettingsMessage::RecordingsFolderChosen(None) => Task::none(),
            SettingsMessage::Save => self.handle_save_settings(),
            SettingsMessage::Cancel => self.close_settings_window(),
        }
    }

    fn close_settings_window(&self) -> Task<OrchestratorMessage> {
        self.settings_window_id
            .map(window::close)
            .unwrap_or_else(Task::none)
    }

    fn apply_draft_change(&mut self, change: SettingsChange) -> Task<OrchestratorMessage> {
        let _ = self.log_debug_event(
            EVENT_SETTINGS_DRAFT_CHANGED,
            serde_json::json!({"change": format!("{:?}", change)}),
        );
        let _ = self.update_settings_draft(|draft| {
            let _ = apply_settings_change(draft, change);
        });
        if let Some(state) = self.settings_view_state_mut() {
            state.validation_error = None;
        }
        Task::none()
    }

    fn choose_recordings_folder(&self) -> Task<OrchestratorMessage> {
        let current_folder =
            std::path::PathBuf::from(&self.get_settings_for_rendering().recording_save_location);
        Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .set_directory(current_folder)
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |folder| OrchestratorMessage::Settings(SettingsMessage::RecordingsFolderChosen(folder)),
        )
    }

    pub(super) fn handle_key_pressed_in_window(
        &mut self,
        window_id: Id,
        modifiers: PressedModifiers,
        physical_code: String,
    ) -> Task<OrchestratorMessage> {
        if self.handle_player_key(window_id, &physical_code) {
            return Task::none();
        }
        if self.settings_window_id != Some(window_id) {
            return Task::none();
        }
        let Some(action) = self
            .settings_view_state_mut()
            .and_then(|state| state.capturing_shortcut)
        else {
            return Task::none();
        };
        match build_accelerator(modifiers, &physical_code) {
            Ok(accelerator) => {
                let _ = self
                    .settings_view_state_mut()
                    .map(|state| state.finish_shortcut_capture());
                self.apply_draft_change(SettingsChange::Shortcut(action, accelerator))
            }
            Err(outcome) => {
                if let Some(state) = self.settings_view_state_mut() {
                    state.shortcut_hint = Some(outcome);
                }
                Task::none()
            }
        }
    }
}
