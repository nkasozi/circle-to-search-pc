use super::*;
use crate::adapters::{GoogleLensSearchProvider, ImgbbImageHostingService};

impl AppOrchestrator {
    pub(super) fn handle_open_settings(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event("settings_window_opening", serde_json::json!({}));

        if let Some(id) = self.settings_window_id {
            self.log_info_event(
                "settings_window_already_open",
                serde_json::json!({"window_id": format!("{:?}", id)}),
            );
            return window::gain_focus(id);
        }

        let (id, task) = window::open(window::Settings {
            size: Size::new(500.0, 800.0),
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });

        self.settings_window_id = Some(id);
        self.begin_settings_edit();
        self.windows.insert(id, AppWindow::Settings);
        self.log_info_event(
            "settings_window_created",
            serde_json::json!({"window_id": format!("{:?}", id)}),
        );

        task.discard()
    }

    pub(super) fn validate_image_hosting_settings(settings: &UserSettings) -> Result<(), String> {
        if settings.image_hosting_provider_url.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_URL_EMPTY.to_string());
        }

        if reqwest::Url::parse(settings.image_hosting_provider_url.trim()).is_err() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_URL_INVALID.to_string());
        }

        if settings.image_hosting_public_key_name.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_KEY_NAME_EMPTY.to_string());
        }

        if settings.image_hosting_public_key_value.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_KEY_EMPTY.to_string());
        }

        if settings.image_hosting_expiration_seconds.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_EXPIRATION_EMPTY.to_string());
        }

        if settings
            .image_hosting_expiration_seconds
            .trim()
            .parse::<u64>()
            .map_or(true, |expiration_seconds| expiration_seconds == 0)
        {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_EXPIRATION_INVALID.to_string());
        }

        Ok(())
    }

    pub(super) fn should_rebuild_search_provider(
        previous_settings: &UserSettings,
        next_settings: &UserSettings,
    ) -> bool {
        next_settings.image_search_url_template != previous_settings.image_search_url_template
            || next_settings.image_hosting_provider_url
                != previous_settings.image_hosting_provider_url
            || next_settings.image_hosting_auth_mode != previous_settings.image_hosting_auth_mode
            || next_settings.image_hosting_public_key_name
                != previous_settings.image_hosting_public_key_name
            || next_settings.image_hosting_public_key_value
                != previous_settings.image_hosting_public_key_value
            || next_settings.image_hosting_expiration_seconds
                != previous_settings.image_hosting_expiration_seconds
            || next_settings.image_hosting_http_method
                != previous_settings.image_hosting_http_method
            || next_settings.image_hosting_image_field_name
                != previous_settings.image_hosting_image_field_name
    }

    pub(super) fn handle_save_settings(&mut self) -> Task<OrchestratorMessage> {
        let settings_for_validation = match &self.settings_edit_state {
            SettingsEditState::Editing(settings) => settings,
            SettingsEditState::Closed => {
                self.log_error_event("settings_save_without_active_editor", serde_json::json!({}));
                self.status = global_constants::STATUS_SETTINGS_EDITOR_NOT_ACTIVE.to_string();
                return Task::none();
            }
        };

        if let Err(validation_error) =
            Self::validate_image_hosting_settings(settings_for_validation)
        {
            self.log_error_event(
                "settings_validation_failed",
                serde_json::json!({"error": validation_error}),
            );
            self.status = format!(
                "{}{}",
                global_constants::STATUS_SETTINGS_SAVE_FAILED_PREFIX,
                validation_error
            );
            return Task::none();
        }

        if let Some(next_settings) = self.take_settings_draft() {
            let hotkey_changed = next_settings.capture_hotkey != self.settings.capture_hotkey;
            let search_provider_changed =
                Self::should_rebuild_search_provider(&self.settings, &next_settings);

            self.settings = next_settings.clone();

            if let Err(save_error) = self.settings.save() {
                self.log_error_event(
                    "settings_save_failed",
                    serde_json::json!({"error": save_error.to_string()}),
                );
                self.status = format!(
                    "{}{}",
                    global_constants::STATUS_SETTINGS_SAVE_FAILED_PREFIX,
                    save_error
                );
            } else {
                self.log_info_event("settings_saved", serde_json::json!({}));
                self.status = global_constants::STATUS_SETTINGS_SAVED.to_string();

                if search_provider_changed {
                    let _ = self.rebuild_reverse_image_search_provider();
                }

                if hotkey_changed {
                    self.log_info_event(
                        "settings_hotkey_changed_restart_required",
                        serde_json::json!({}),
                    );
                    return Task::done(OrchestratorMessage::RestartApp);
                }
            }
        }

        if let Some(id) = self.settings_window_id {
            return window::close(id);
        }

        Task::none()
    }

    fn rebuild_reverse_image_search_provider(&mut self) -> bool {
        let image_hosting_service =
            Arc::new(ImgbbImageHostingService::from_user_settings(&self.settings));
        self.reverse_image_search_provider = Arc::new(GoogleLensSearchProvider::new(
            image_hosting_service,
            self.settings.image_search_url_template.clone(),
        ));
        self.log_info_event("search_provider_rebuilt", serde_json::json!({}));

        true
    }
}
