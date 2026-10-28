use std::path::PathBuf;

use crate::core::models::GlobalHotkeyAction;

mod changes;
mod controls;
mod general_tab;
mod layout;
mod recording_tab;
mod search_tab;
mod shortcut_capture;

pub use changes::{apply_settings_change, SettingsChange};
pub use layout::render_settings;
pub use shortcut_capture::{build_accelerator, PressedModifiers, ShortcutCaptureOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Recording,
    Search,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    SelectTab(SettingsTab),
    Change(SettingsChange),
    StartShortcutCapture(GlobalHotkeyAction),
    CancelShortcutCapture,
    ChooseRecordingsFolder,
    RecordingsFolderChosen(Option<PathBuf>),
    Save,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SettingsViewState {
    pub tab: SettingsTab,
    pub capturing_shortcut: Option<GlobalHotkeyAction>,
    pub shortcut_hint: Option<ShortcutCaptureOutcome>,
    pub validation_error: Option<String>,
}

impl SettingsViewState {
    pub fn select_tab(&mut self, tab: SettingsTab) -> bool {
        let changed = self.tab != tab;
        self.tab = tab;
        self.capturing_shortcut = None;
        changed
    }

    pub fn begin_shortcut_capture(&mut self, action: GlobalHotkeyAction) -> bool {
        self.capturing_shortcut = Some(action);
        self.shortcut_hint = None;
        true
    }

    pub fn finish_shortcut_capture(&mut self) -> Option<GlobalHotkeyAction> {
        self.shortcut_hint = None;
        self.capturing_shortcut.take()
    }
}
