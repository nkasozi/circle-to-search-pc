use std::collections::HashMap;
use std::sync::Arc;

use iced::widget::{container, text, Space};
use iced::window::{self, Id};
use iced::{Element, Point, Rectangle, Size, Task};
use mouse_position::mouse_position::Mouse;

use crate::core::interfaces::adapters::{OcrService, ReverseImageSearchProvider};
use crate::core::interfaces::ports::{FileLauncher, MediaPlayer, RecordingLibrary, ScreenCapturer};
use crate::core::models::{
    CaptureBuffer, GlobalHotkeyAction, HotkeyRegistrationFailure, OcrResult, RecordingSummary,
    ScreenRegion, UserSettings, WindowInfo,
};
use crate::core::orchestrators::recording_flow::{
    RecordingDependencies, RecordingFlow, RecordingMessage,
};
use crate::global_constants;
use crate::ports::TrayEvent;
use crate::presentation::{CaptureView, CaptureViewMessage, OnboardingMessage, OnboardingView};
use crate::presentation::{WindowPickerMessage, WindowPickerView};

const CORRELATION_ID_STARTUP: &str = "startup";
pub use message::OrchestratorMessage;

const CORRELATION_ID_ORCHESTRATOR_PREFIX: &str = "orchestrator-";

mod capture;
mod dispatch;
mod hidden_window;
mod home;
mod hotkeys;
mod image_actions;
mod interactive_ocr;
mod logging;
mod message;
mod onboarding;
mod player;
mod recording;
mod settings;
mod settings_draft;
mod ui;
mod window_appearance;
mod window_lifecycle;
mod window_picker;
mod window_rendering;

pub enum AppWindow {
    Main,
    CaptureOverlay(CaptureView),
    InteractiveOcr(crate::presentation::InteractiveOcrView),
    Settings(crate::presentation::settings_view::SettingsViewState),
    Onboarding(OnboardingView),
    Hidden,
    WindowPicker(WindowPickerView),
    Player(Box<player::PlayerWindow>),
}

#[derive(Debug, Clone)]
pub enum SettingsEditState {
    Closed,
    Editing(UserSettings),
}

pub struct AppDependencies {
    pub screen_capturer: Arc<dyn ScreenCapturer>,
    pub ocr_service: Arc<dyn OcrService>,
    pub reverse_image_search_provider: Arc<dyn ReverseImageSearchProvider>,
    pub recording_library: Arc<dyn RecordingLibrary>,
    pub file_launcher: Arc<dyn FileLauncher>,
    pub media_player: Arc<dyn MediaPlayer>,
    pub recording: RecordingDependencies,
}

pub struct AppOrchestrator {
    screen_capturer: Arc<dyn ScreenCapturer>,
    ocr_service: Arc<dyn OcrService>,
    reverse_image_search_provider: Arc<dyn ReverseImageSearchProvider>,
    recording_library: Arc<dyn RecordingLibrary>,
    file_launcher: Arc<dyn FileLauncher>,
    media_player: Arc<dyn MediaPlayer>,
    recording_flow: RecordingFlow,
    recent_recordings: Vec<RecordingSummary>,
    windows: HashMap<Id, AppWindow>,
    main_window_id: Option<Id>,
    onboarding_window_id: Option<Id>,
    hidden_window_id: Option<Id>,
    window_picker_window_id: Option<Id>,
    status: String,
    settings: UserSettings,
    settings_window_id: Option<Id>,
    settings_edit_state: SettingsEditState,
    pending_draw_strokes: Option<Vec<crate::presentation::DrawStroke>>,
    current_correlation_id: String,
}

impl AppOrchestrator {
    pub fn build(dependencies: AppDependencies, settings: UserSettings) -> Self {
        let recording_flow = RecordingFlow::build(
            dependencies.recording,
            settings.recording_preferences,
            std::path::PathBuf::from(&settings.recording_save_location),
        );
        Self {
            screen_capturer: dependencies.screen_capturer,
            ocr_service: dependencies.ocr_service,
            reverse_image_search_provider: dependencies.reverse_image_search_provider,
            recording_library: dependencies.recording_library,
            file_launcher: dependencies.file_launcher,
            media_player: dependencies.media_player,
            recording_flow,
            recent_recordings: Vec::new(),
            windows: HashMap::new(),
            main_window_id: None,
            onboarding_window_id: None,
            hidden_window_id: None,
            window_picker_window_id: None,
            status: global_constants::STATUS_INITIALIZING.to_string(),
            settings,
            settings_window_id: None,
            settings_edit_state: SettingsEditState::Closed,
            pending_draw_strokes: None,
            current_correlation_id: CORRELATION_ID_STARTUP.to_string(),
        }
    }

    fn refresh_correlation_id(&mut self) -> bool {
        let now = std::time::SystemTime::now();
        let elapsed_microseconds = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        self.current_correlation_id = format!(
            "{}{}",
            CORRELATION_ID_ORCHESTRATOR_PREFIX, elapsed_microseconds
        );
        true
    }

    pub(super) fn current_correlation_id(&self) -> String {
        self.current_correlation_id.clone()
    }

    pub fn current_settings(&self) -> &UserSettings {
        &self.settings
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod player_tests;
