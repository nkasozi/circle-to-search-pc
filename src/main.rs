#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod app;
mod app_theme;
mod core;
mod global_constants;
mod ports;
mod presentation;
mod user_settings;
mod utils;

use iced::daemon;

fn main() -> iced::Result {
    env_logger::init();

    if !utils::ensure_single_instance() {
        log::error!("[MAIN] Failed to ensure single instance");
    }

    daemon(
        app::CircleApp::build,
        app::CircleApp::handle_update,
        app::CircleApp::render_view,
    )
    .subscription(app::CircleApp::handle_subscription)
    .run()
}
