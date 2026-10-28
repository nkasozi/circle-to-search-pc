use super::*;

impl AppOrchestrator {
    pub(super) fn begin_settings_edit(&mut self) -> bool {
        self.settings_edit_state = SettingsEditState::Editing(self.settings.clone());
        true
    }

    pub(super) fn discard_settings_edit(&mut self) -> bool {
        self.settings_edit_state = SettingsEditState::Closed;
        true
    }

    pub(super) fn get_settings_for_rendering(&self) -> &UserSettings {
        match &self.settings_edit_state {
            SettingsEditState::Closed => &self.settings,
            SettingsEditState::Editing(settings) => settings,
        }
    }

    pub(super) fn update_settings_draft(
        &mut self,
        update_function: impl FnOnce(&mut UserSettings),
    ) -> bool {
        match &mut self.settings_edit_state {
            SettingsEditState::Closed => false,
            SettingsEditState::Editing(settings) => {
                update_function(settings);
                true
            }
        }
    }

    pub(super) fn take_settings_draft(&mut self) -> Option<UserSettings> {
        match std::mem::replace(&mut self.settings_edit_state, SettingsEditState::Closed) {
            SettingsEditState::Closed => None,
            SettingsEditState::Editing(settings) => Some(settings),
        }
    }
}
