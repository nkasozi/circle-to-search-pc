use auto_launch::AutoLaunch;

const LOG_TAG_AUTO_LAUNCH: &str = "[AUTO_LAUNCH]";
const APP_NAME: &str = "Circle to Search";
const LAUNCH_AT_LOGIN_STATUS_CHECKED_EVENT: &str = "launch_at_login_status_checked";
const LAUNCH_AT_LOGIN_UPDATE_REQUESTED_EVENT: &str = "launch_at_login_update_requested";
const AUTO_LAUNCH_INSTANCE_CREATION_FAILED_EVENT: &str = "auto_launch_instance_creation_failed";
const LAUNCH_AT_LOGIN_UPDATED_EVENT: &str = "launch_at_login_updated";
const LAUNCH_AT_LOGIN_UPDATE_FAILED_EVENT: &str = "launch_at_login_update_failed";
const AUTO_LAUNCH_EXECUTABLE_PATH_RESOLVED_EVENT: &str = "auto_launch_executable_path_resolved";
#[cfg(target_os = "macos")]
const AUTO_LAUNCH_UNAVAILABLE_OUTSIDE_APP_BUNDLE_EVENT: &str =
    "auto_launch_unavailable_outside_app_bundle";
#[cfg(target_os = "macos")]
const AUTO_LAUNCH_APP_PATH_RESOLVED_EVENT: &str = "auto_launch_app_path_resolved";
#[cfg(target_os = "macos")]
const MACOS_APP_BUNDLE_EXECUTABLE_MARKER: &str = ".app/Contents/MacOS/";
#[cfg(target_os = "macos")]
const MACOS_APP_BUNDLE_EXTENSION: &str = ".app";

pub fn is_launch_at_login_enabled() -> bool {
    match get_auto_launch() {
        Some(auto_launch) => {
            let enabled = auto_launch.is_enabled().unwrap_or(false);
            log::debug!(
                "{} {}",
                LOG_TAG_AUTO_LAUNCH,
                serde_json::json!({"event": LAUNCH_AT_LOGIN_STATUS_CHECKED_EVENT, "enabled": enabled})
            );
            enabled
        }
        None => false,
    }
}

pub fn set_launch_at_login(enabled: bool) -> bool {
    log::info!(
        "{} {}",
        LOG_TAG_AUTO_LAUNCH,
        serde_json::json!({"event": LAUNCH_AT_LOGIN_UPDATE_REQUESTED_EVENT, "enabled": enabled})
    );

    let Some(auto_launch) = get_auto_launch() else {
        log::error!(
            "{} {}",
            LOG_TAG_AUTO_LAUNCH,
            serde_json::json!({"event": AUTO_LAUNCH_INSTANCE_CREATION_FAILED_EVENT})
        );
        return false;
    };

    apply_launch_at_login(&auto_launch, enabled)
}

fn apply_launch_at_login(auto_launch: &AutoLaunch, enabled: bool) -> bool {
    let result = match enabled {
        true => auto_launch.enable(),
        false => auto_launch.disable(),
    };

    match result {
        Ok(()) => {
            log::info!(
                "{} {}",
                LOG_TAG_AUTO_LAUNCH,
                serde_json::json!({"event": LAUNCH_AT_LOGIN_UPDATED_EVENT, "enabled": enabled})
            );
            true
        }
        Err(error) => {
            log::error!(
                "{} {}",
                LOG_TAG_AUTO_LAUNCH,
                serde_json::json!({"event": LAUNCH_AT_LOGIN_UPDATE_FAILED_EVENT, "enabled": enabled, "error": error.to_string()})
            );
            false
        }
    }
}

fn get_auto_launch() -> Option<AutoLaunch> {
    let current_executable = std::env::current_exe().ok()?;
    let executable_path = current_executable.to_str()?;

    log::debug!(
        "{} {}",
        LOG_TAG_AUTO_LAUNCH,
        serde_json::json!({"event": AUTO_LAUNCH_EXECUTABLE_PATH_RESOLVED_EVENT, "executable_path": executable_path})
    );

    #[cfg(target_os = "macos")]
    {
        if !executable_path.contains(MACOS_APP_BUNDLE_EXECUTABLE_MARKER) {
            log::warn!(
                "{} {}",
                LOG_TAG_AUTO_LAUNCH,
                serde_json::json!({"event": AUTO_LAUNCH_UNAVAILABLE_OUTSIDE_APP_BUNDLE_EVENT, "executable_path": executable_path})
            );
            return None;
        }

        let app_path = find_macos_app_bundle_path(executable_path);
        log::debug!(
            "{} {}",
            LOG_TAG_AUTO_LAUNCH,
            serde_json::json!({"event": AUTO_LAUNCH_APP_PATH_RESOLVED_EVENT, "app_path": app_path})
        );

        Some(AutoLaunch::new(APP_NAME, &app_path, false, &[] as &[&str]))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Some(AutoLaunch::new(APP_NAME, executable_path, &[] as &[&str]))
    }
}

#[cfg(target_os = "macos")]
fn find_macos_app_bundle_path(executable_path: &str) -> String {
    match executable_path.split_once(MACOS_APP_BUNDLE_EXECUTABLE_MARKER) {
        Some((bundle_prefix, _)) => format!("{}{}", bundle_prefix, MACOS_APP_BUNDLE_EXTENSION),
        None => executable_path.to_string(),
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::find_macos_app_bundle_path;

    #[test]
    fn test_find_macos_app_bundle_path_with_app_bundle() {
        let exe_path = "/Applications/Circle to Search.app/Contents/MacOS/circle-to-search-pc";
        let result = find_macos_app_bundle_path(exe_path);
        assert_eq!(result, "/Applications/Circle to Search.app");
    }

    #[test]
    fn test_find_macos_app_bundle_path_without_app_bundle() {
        let exe_path = "/usr/local/bin/circle-to-search-pc";
        let result = find_macos_app_bundle_path(exe_path);
        assert_eq!(result, "/usr/local/bin/circle-to-search-pc");
    }
}
