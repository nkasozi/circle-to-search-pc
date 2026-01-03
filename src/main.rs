#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod app;
mod app_theme;
mod core;
mod global_constants;
mod ports;
mod presentation;
mod system_tray;
mod user_settings;
mod utils;

#[cfg(test)]
mod app_theme_tests;
#[cfg(test)]
mod system_tray_tests;

use iced::daemon;

fn main() -> iced::Result {
    env_logger::init();

    if !utils::ensure_single_instance() {
        log::error!("[MAIN] Failed to ensure single instance");
    }

    #[cfg(target_os = "macos")]
    {
        use tray_icon::TrayIconEvent;
        TrayIconEvent::set_event_handler(Some(|_event| {}));
    }

    daemon(
        app::CircleApp::build,
        app::CircleApp::handle_update,
        app::CircleApp::render_view,
    )
    .subscription(app::CircleApp::handle_subscription)
    .run()
}
