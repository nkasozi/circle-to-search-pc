use auto_launch::AutoLaunch;

const LOG_TAG_AUTO_LAUNCH: &str = "[AUTO_LAUNCH]";
const APP_NAME: &str = "Circle to Search";

pub fn is_launch_at_login_enabled() -> bool {
    match get_auto_launch() {
        Some(auto_launch) => {
            let enabled = auto_launch.is_enabled().unwrap_or(false);
            log::debug!(
                "{} Launch at login enabled: {}",
                LOG_TAG_AUTO_LAUNCH,
                enabled
            );
            enabled
        }
        None => false,
    }
}

pub fn set_launch_at_login(enabled: bool) -> bool {
    log::info!(
        "{} Setting launch at login to: {}",
        LOG_TAG_AUTO_LAUNCH,
        enabled
    );

    match get_auto_launch() {
        Some(auto_launch) => {
            let result = if enabled {
                auto_launch.enable()
            } else {
                auto_launch.disable()
            };

            match result {
                Ok(()) => {
                    log::info!(
                        "{} Successfully {} launch at login",
                        LOG_TAG_AUTO_LAUNCH,
                        if enabled { "enabled" } else { "disabled" }
                    );
                    true
                }
                Err(error) => {
                    log::error!(
                        "{} Failed to {} launch at login: {}",
                        LOG_TAG_AUTO_LAUNCH,
                        if enabled { "enable" } else { "disable" },
                        error
                    );
                    false
                }
            }
        }
        None => {
            log::error!(
                "{} Could not create AutoLaunch instance",
                LOG_TAG_AUTO_LAUNCH
            );
            false
        }
    }
}

fn get_auto_launch() -> Option<AutoLaunch> {
    let current_exe = std::env::current_exe().ok()?;
    let exe_path = current_exe.to_str()?;

    log::debug!("{} Executable path: {}", LOG_TAG_AUTO_LAUNCH, exe_path);

    #[cfg(target_os = "macos")]
    {
        let app_path = find_macos_app_bundle_path(exe_path);
        log::debug!("{} Using app path: {}", LOG_TAG_AUTO_LAUNCH, app_path);

        Some(AutoLaunch::new(APP_NAME, &app_path, false, &[] as &[&str]))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Some(AutoLaunch::new(APP_NAME, exe_path, &[] as &[&str]))
    }
}

#[cfg(target_os = "macos")]
fn find_macos_app_bundle_path(exe_path: &str) -> String {
    if exe_path.contains(".app/Contents/MacOS/") {
        let parts: Vec<&str> = exe_path.split(".app/Contents/MacOS/").collect();
        if !parts.is_empty() {
            return format!("{}.app", parts[0]);
        }
    }

    exe_path.to_string()
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
