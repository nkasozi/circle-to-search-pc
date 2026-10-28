use super::*;
use crate::core::models::{current_platform_key_symbol_style, format_accelerator_for_display};

const HOTKEY_CONFLICT_STATUS_PREFIX: &str = "Shortcut unavailable: ";
const HOTKEY_CONFLICT_STATUS_SEPARATOR: &str = ", ";
const EVENT_HOTKEYS_APPLIED: &str = "hotkeys_applied";

impl AppOrchestrator {
    pub(super) fn handle_hotkeys_applied(
        &mut self,
        failures: Vec<HotkeyRegistrationFailure>,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(
            EVENT_HOTKEYS_APPLIED,
            serde_json::json!({"failure_count": failures.len()}),
        );
        if failures.is_empty() {
            return Task::none();
        }
        self.status = build_hotkey_failure_status(&failures);
        Task::none()
    }
}

pub(super) fn build_hotkey_failure_status(failures: &[HotkeyRegistrationFailure]) -> String {
    let symbol_style = current_platform_key_symbol_style();
    let formatted_shortcuts: Vec<String> = failures
        .iter()
        .map(|failure| format_accelerator_for_display(&failure.accelerator, symbol_style))
        .collect();
    format!(
        "{}{}",
        HOTKEY_CONFLICT_STATUS_PREFIX,
        formatted_shortcuts.join(HOTKEY_CONFLICT_STATUS_SEPARATOR)
    )
}
