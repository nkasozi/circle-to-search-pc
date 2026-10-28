use super::*;
use crate::adapters::{auto_launch, GoogleLensSearchProvider, ImgbbImageHostingService};

const EVENT_SETTINGS_VALIDATION_FAILED: &str = "settings_validation_failed";
const EVENT_SETTINGS_SAVE_FAILED: &str = "settings_save_failed";
const EVENT_SETTINGS_SAVED: &str = "settings_saved";
const EVENT_SEARCH_PROVIDER_REBUILT: &str = "search_provider_rebuilt";

impl AppOrchestrator {
    pub(super) fn handle_save_settings(&mut self) -> Task<OrchestratorMessage> {
        let validation = match &self.settings_edit_state {
            SettingsEditState::Editing(draft) => Self::validate_image_hosting_settings(draft),
            SettingsEditState::Closed => return Task::none(),
        };
        if let Err(validation_error) = validation {
            return self.report_settings_validation_error(validation_error);
        }
        let Some(next_settings) = self.take_settings_draft() else {
            return Task::none();
        };
        let follow_up = self.commit_settings(next_settings);
        let close_task = self.close_settings_window();
        Task::batch([follow_up, close_task])
    }

    fn report_settings_validation_error(
        &mut self,
        validation_error: String,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_error_event(
            EVENT_SETTINGS_VALIDATION_FAILED,
            serde_json::json!({"error": validation_error}),
        );
        if let Some(state) = self.settings_view_state_mut() {
            state.validation_error = Some(validation_error);
        }
        Task::none()
    }

    fn commit_settings(&mut self, next_settings: UserSettings) -> Task<OrchestratorMessage> {
        let previous_settings = std::mem::replace(&mut self.settings, next_settings);
        if let Err(save_error) = self.settings.save() {
            let _ = self.log_error_event(
                EVENT_SETTINGS_SAVE_FAILED,
                serde_json::json!({"error": save_error.to_string()}),
            );
            self.status = format!(
                "{}{}",
                global_constants::STATUS_SETTINGS_SAVE_FAILED_PREFIX,
                save_error
            );
            return Task::none();
        }
        self.status = global_constants::STATUS_SETTINGS_SAVED.to_string();
        let _ = self.apply_committed_settings_side_effects(&previous_settings);
        let hotkeys_changed = previous_settings.capture_hotkey != self.settings.capture_hotkey
            || previous_settings.recording_hotkey != self.settings.recording_hotkey;
        let _ = self.log_info_event(
            EVENT_SETTINGS_SAVED,
            serde_json::json!({"hotkeys_changed": hotkeys_changed}),
        );
        let reapply_hotkeys_task = match hotkeys_changed {
            true => Task::done(OrchestratorMessage::ReapplyHotkeys),
            false => Task::none(),
        };
        Task::batch([reapply_hotkeys_task, self.refresh_recent_recordings()])
    }

    fn apply_committed_settings_side_effects(&mut self, previous_settings: &UserSettings) -> bool {
        if Self::should_rebuild_search_provider(previous_settings, &self.settings) {
            let _ = self.rebuild_reverse_image_search_provider();
        }
        if previous_settings.launch_at_login != self.settings.launch_at_login {
            let _ = auto_launch::set_launch_at_login(self.settings.launch_at_login);
        }
        let _ = self.recording_flow.apply_settings(
            self.settings.recording_preferences,
            std::path::PathBuf::from(&self.settings.recording_save_location),
        );
        true
    }

    fn rebuild_reverse_image_search_provider(&mut self) -> bool {
        let image_hosting_service =
            Arc::new(ImgbbImageHostingService::from_user_settings(&self.settings));
        self.reverse_image_search_provider = Arc::new(GoogleLensSearchProvider::new(
            image_hosting_service,
            self.settings.image_search_url_template.clone(),
        ));
        let _ = self.log_info_event(EVENT_SEARCH_PROVIDER_REBUILT, serde_json::json!({}));
        true
    }
}
