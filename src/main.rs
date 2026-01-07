#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod core;
mod global_constants;
mod infrastructure;
mod ports;
mod presentation;

use iced::daemon;

fn main() -> iced::Result {
    env_logger::init();

    log::info!("[MAIN] Starting Circle to Search application");

    let lock_file_path = infrastructure::utils::get_default_lock_file_path();
    if !infrastructure::utils::ensure_single_instance_using_lock_file(&lock_file_path) {
        log::error!("[MAIN] Failed to ensure single instance");
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
    .subscription(core::orchestrators::app::CircleApp::handle_subscription)
    .run()
}
