#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod core;
mod global_constants;
mod infrastructure;
mod ports;
mod presentation;

use iced::daemon;

const LOG_TAG: &str = "[MAIN]";
const APPLICATION_STARTING_EVENT: &str = "application_starting";
const SINGLE_INSTANCE_ENFORCEMENT_FAILED_EVENT: &str = "single_instance_enforcement_failed";

fn main() -> iced::Result {
    env_logger::init();

    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": APPLICATION_STARTING_EVENT, "application_title": global_constants::APPLICATION_TITLE})
    );

    let lock_file_path = infrastructure::utils::get_default_lock_file_path();
    if !infrastructure::utils::ensure_single_instance_using_lock_file(&lock_file_path) {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SINGLE_INSTANCE_ENFORCEMENT_FAILED_EVENT, "lock_file_path": lock_file_path.display().to_string()})
        );
    }

    #[cfg(target_os = "macos")]
    {
        use tray_icon::TrayIconEvent;
        TrayIconEvent::set_event_handler(Some(|_event| {}));
    }

    daemon(
        core::orchestrators::app::CircleApp::build,
        core::orchestrators::app::CircleApp::handle_update,
        core::orchestrators::app::CircleApp::render_view,
    )
    .title(core::orchestrators::app::CircleApp::window_title)
    .theme(core::orchestrators::app::CircleApp::window_theme)
    .subscription(core::orchestrators::app::CircleApp::handle_subscription)
    .run()
}
