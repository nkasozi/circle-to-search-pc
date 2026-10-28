use std::sync::Arc;

use iced::Task;

use super::constants::{
    EVENT_LAUNCHING_HIDDEN, EVENT_OCR_SERVICE_INITIALIZATION_FAILED, EVENT_OCR_SERVICE_INITIALIZED,
    EVENT_SETTINGS_LOAD_FAILED, EVENT_SHOWING_MAIN_WINDOW, EVENT_SHOWING_ONBOARDING,
    EVENT_SYSTEM_TRAY_INITIALIZATION_FAILED, EVENT_SYSTEM_TRAY_INITIALIZED, LOG_TAG,
};
use super::placeholder_ocr::DummyOcrService;
use crate::adapters::{
    DesktopFileLauncher, FileSystemRecordingLibrary, GoogleLensSearchProvider,
    ImgbbImageHostingService, TesseractOcrService,
};
use crate::core::interfaces::adapters::OcrService;
use crate::core::models::{HotkeyRegistrationFailure, UserSettings};
use crate::core::orchestrators::app_composition::{
    build_media_player, build_recording_dependencies,
};
use crate::core::orchestrators::app_orchestrator::{
    AppDependencies, AppOrchestrator, OrchestratorMessage,
};
use crate::ports::{SystemTray, TrayShortcuts, XcapScreenCapturer};

pub fn load_settings() -> UserSettings {
    UserSettings::load().unwrap_or_else(|error| {
        log::warn!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_SETTINGS_LOAD_FAILED, "error": error.to_string()})
        );
        UserSettings::default()
    })
}

pub fn build_orchestrator(settings: UserSettings) -> AppOrchestrator {
    let image_hosting_service = Arc::new(ImgbbImageHostingService::from_user_settings(&settings));
    let reverse_image_search_provider = Arc::new(GoogleLensSearchProvider::new(
        image_hosting_service,
        settings.image_search_url_template.clone(),
    ));
    AppOrchestrator::build(
        AppDependencies {
            screen_capturer: Arc::new(XcapScreenCapturer::initialize()),
            ocr_service: Arc::new(DummyOcrService),
            reverse_image_search_provider,
            recording_library: Arc::new(FileSystemRecordingLibrary),
            file_launcher: Arc::new(DesktopFileLauncher),
            media_player: build_media_player(),
            recording: build_recording_dependencies(),
        },
        settings,
    )
}

pub fn build_tray(orchestrator: &AppOrchestrator) -> Option<SystemTray> {
    let tray_shortcuts = TrayShortcuts {
        capture_hotkey: &orchestrator.current_settings().capture_hotkey,
        recording_hotkey: &orchestrator.current_settings().recording_hotkey,
    };
    match SystemTray::build(tray_shortcuts) {
        Ok(tray) => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": EVENT_SYSTEM_TRAY_INITIALIZED})
            );
            Some(tray)
        }
        Err(error) => {
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": EVENT_SYSTEM_TRAY_INITIALIZATION_FAILED, "error": error.to_string()})
            );
            None
        }
    }
}

fn initialize_ocr_service() -> OrchestratorMessage {
    match TesseractOcrService::build() {
        Ok(service) => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": EVENT_OCR_SERVICE_INITIALIZED})
            );
            OrchestratorMessage::OcrServiceReady(Arc::new(service) as Arc<dyn OcrService>)
        }
        Err(error) => {
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": EVENT_OCR_SERVICE_INITIALIZATION_FAILED, "error": error.to_string()})
            );
            OrchestratorMessage::OcrServiceFailed(error.to_string())
        }
    }
}

pub fn choose_launch_message(
    needs_onboarding: bool,
    show_home_on_launch: bool,
) -> Option<OrchestratorMessage> {
    let (event, launch_message) = match (needs_onboarding, show_home_on_launch) {
        (true, _) => (
            EVENT_SHOWING_ONBOARDING,
            Some(OrchestratorMessage::OpenOnboarding),
        ),
        (false, true) => (
            EVENT_SHOWING_MAIN_WINDOW,
            Some(OrchestratorMessage::OpenMainWindow),
        ),
        (false, false) => (EVENT_LAUNCHING_HIDDEN, None),
    };
    log::info!("{} {}", LOG_TAG, serde_json::json!({"event": event}));
    launch_message
}

pub fn build_startup_tasks(
    hotkey_failures: Vec<HotkeyRegistrationFailure>,
    launch_message: Option<OrchestratorMessage>,
) -> Vec<Task<OrchestratorMessage>> {
    let mut tasks = vec![
        Task::done(OrchestratorMessage::CreateHiddenWindow),
        Task::future(async { initialize_ocr_service() }),
        Task::done(OrchestratorMessage::HotkeysApplied(hotkey_failures)),
    ];
    tasks.extend(launch_message.map(Task::done));
    tasks
}
