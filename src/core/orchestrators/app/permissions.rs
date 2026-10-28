pub fn check_all_permissions_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        use super::constants::{EVENT_PERMISSION_CHECK, LOG_TAG};
        use crate::adapters::macos_permissions::macos::check_screen_recording_permission;

        let screen_recording_granted = check_screen_recording_permission();
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_PERMISSION_CHECK, "screen_recording": screen_recording_granted})
        );
        screen_recording_granted
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}
