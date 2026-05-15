use std::collections::HashMap;
use std::sync::Arc;

use iced::widget::{container, text, Space};
use iced::window::{self, Id};
use iced::{Element, Point, Rectangle, Size, Task};
use mouse_position::mouse_position::Mouse;

use crate::core::interfaces::adapters::{OcrService, ReverseImageSearchProvider};
use crate::core::interfaces::ports::{MousePositionProvider, ScreenCapturer};
use crate::core::models::{
    CaptureBuffer, ImageHostingAuthMode, ImageUploadHttpMethod, OcrResult, ScreenRegion, ThemeMode,
    UserSettings, WindowInfo,
};
use crate::global_constants;
use crate::ports::{GlobalKeyboardEvent, TrayEvent};
use crate::presentation::app_theme;
use crate::presentation::{CaptureView, CaptureViewMessage, OnboardingMessage, OnboardingView};
use crate::presentation::{WindowPickerMessage, WindowPickerView};

const CORRELATION_ID_STARTUP: &str = "startup";
const CORRELATION_ID_ORCHESTRATOR_PREFIX: &str = "orchestrator-";

mod capture;
mod image_actions;
mod interactive_ocr;
mod onboarding;
mod settings;
mod ui;
mod window_lifecycle;
mod window_picker;

pub enum AppWindow {
    Main,
    CaptureOverlay(CaptureView),
    InteractiveOcr(crate::presentation::InteractiveOcrView),
    Settings,
    Onboarding(OnboardingView),
    Hidden,
    WindowPicker(WindowPickerView),
}

#[derive(Debug, Clone)]
pub enum SettingsEditState {
    Closed,
    Editing(UserSettings),
}

pub struct AppOrchestrator {
    screen_capturer: Arc<dyn ScreenCapturer>,
    #[allow(dead_code)]
    mouse_provider: Arc<dyn MousePositionProvider>,
    ocr_service: Arc<dyn OcrService>,
    reverse_image_search_provider: Arc<dyn ReverseImageSearchProvider>,
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

#[derive(Clone)]
pub enum OrchestratorMessage {
    #[allow(dead_code)]
    OpenMainWindow,
    CreateHiddenWindow,
    CaptureScreen,
    PerformCapture,
    OpenCaptureOverlay(i32, i32, CaptureBuffer),
    CaptureError(String),
    CaptureOverlayMessage(Id, CaptureViewMessage),
    ConfirmSelection(Id),
    ShowCroppedImage(CaptureBuffer, Rectangle),
    ProcessOcr(Id, CaptureBuffer),
    OcrComplete(Id, Result<OcrResult, String>),
    OcrServiceReady(Arc<dyn OcrService>),
    OcrServiceFailed(String),
    InteractiveOcrMessage(Id, crate::presentation::InteractiveOcrMessage),
    PerformImageSearch(Id, CaptureBuffer, Option<String>),
    SpinnerTick,
    #[allow(dead_code)]
    CloseWindow(Id),
    WindowClosed(Id),
    WindowFocused(Id),
    Keyboard(GlobalKeyboardEvent),
    OpenSettings,
    UpdateSearchUrl(String),
    UpdateImageHostingProviderUrl(String),
    UpdateImageHostingAuthMode(ImageHostingAuthMode),
    UpdateImageHostingPublicKeyName(String),
    UpdateImageHostingPublicKeyValue(String),
    UpdateImageHostingExpirationSeconds(String),
    UpdateImageHostingHttpMethod(ImageUploadHttpMethod),
    UpdateImageHostingImageFieldName(String),
    UpdateHotkey(String),
    UpdateTheme(ThemeMode),
    UpdateSystemTrayMode(bool),
    SaveSettings,
    RestartApp,
    TrayEvent(TrayEvent),
    #[allow(dead_code)]
    HideMainWindow,
    OpenOnboarding,
    OnboardingMsg(Id, OnboardingMessage),
    EnableKeyboardListener,
    CopyImageToClipboard(Id, CaptureBuffer, Vec<crate::presentation::DrawStroke>),
    SaveImageToFile(Id, CaptureBuffer, Vec<crate::presentation::DrawStroke>),
    #[allow(dead_code)]
    OpenWindowPicker,
    WindowPickerMsg(Id, WindowPickerMessage),
    WindowsListLoaded(Id, Vec<WindowInfo>),
    FocusWindowAndCapture(String),
    #[allow(dead_code)]
    CaptureSelectedWindow(u32),
    WindowCaptureComplete(CaptureBuffer),
    WindowCaptureError(String),
}

impl std::fmt::Debug for OrchestratorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorMessage::OpenMainWindow => write!(f, "OpenMainWindow"),
            OrchestratorMessage::CreateHiddenWindow => write!(f, "CreateHiddenWindow"),
            OrchestratorMessage::CaptureScreen => write!(f, "CaptureScreen"),
            OrchestratorMessage::PerformCapture => write!(f, "PerformCapture"),
            OrchestratorMessage::OpenCaptureOverlay(x, y, _) => {
                write!(f, "OpenCaptureOverlay({}, {})", x, y)
            }
            OrchestratorMessage::CaptureError(e) => write!(f, "CaptureError({})", e),
            OrchestratorMessage::CaptureOverlayMessage(id, _) => {
                write!(f, "CaptureOverlayMessage({:?})", id)
            }
            OrchestratorMessage::ConfirmSelection(id) => write!(f, "ConfirmSelection({:?})", id),
            OrchestratorMessage::ShowCroppedImage(_, rect) => {
                write!(f, "ShowCroppedImage({:?})", rect)
            }
            OrchestratorMessage::ProcessOcr(id, _) => write!(f, "ProcessOcr({:?})", id),
            OrchestratorMessage::OcrComplete(id, result) => {
                write!(f, "OcrComplete({:?}, {:?})", id, result.is_ok())
            }
            OrchestratorMessage::OcrServiceReady(_) => write!(f, "OcrServiceReady"),
            OrchestratorMessage::OcrServiceFailed(e) => write!(f, "OcrServiceFailed({})", e),
            OrchestratorMessage::InteractiveOcrMessage(id, _) => {
                write!(f, "InteractiveOcrMessage({:?})", id)
            }
            OrchestratorMessage::PerformImageSearch(id, _, query) => {
                write!(f, "PerformImageSearch({:?}, query={:?})", id, query)
            }
            OrchestratorMessage::SpinnerTick => write!(f, "SpinnerTick"),
            OrchestratorMessage::CloseWindow(id) => write!(f, "CloseWindow({:?})", id),
            OrchestratorMessage::WindowClosed(id) => write!(f, "WindowClosed({:?})", id),
            OrchestratorMessage::WindowFocused(id) => write!(f, "WindowFocused({:?})", id),
            OrchestratorMessage::Keyboard(event) => write!(f, "Keyboard({:?})", event),
            OrchestratorMessage::OpenSettings => write!(f, "OpenSettings"),
            OrchestratorMessage::UpdateSearchUrl(_) => write!(f, "UpdateSearchUrl"),
            OrchestratorMessage::UpdateImageHostingProviderUrl(_) => {
                write!(f, "UpdateImageHostingProviderUrl")
            }
            OrchestratorMessage::UpdateImageHostingAuthMode(_) => {
                write!(f, "UpdateImageHostingAuthMode")
            }
            OrchestratorMessage::UpdateImageHostingPublicKeyName(_) => {
                write!(f, "UpdateImageHostingPublicKeyName")
            }
            OrchestratorMessage::UpdateImageHostingPublicKeyValue(_) => {
                write!(f, "UpdateImageHostingPublicKeyValue")
            }
            OrchestratorMessage::UpdateImageHostingExpirationSeconds(_) => {
                write!(f, "UpdateImageHostingExpirationSeconds")
            }
            OrchestratorMessage::UpdateImageHostingHttpMethod(_) => {
                write!(f, "UpdateImageHostingHttpMethod")
            }
            OrchestratorMessage::UpdateImageHostingImageFieldName(_) => {
                write!(f, "UpdateImageHostingImageFieldName")
            }
            OrchestratorMessage::UpdateHotkey(_) => write!(f, "UpdateHotkey"),
            OrchestratorMessage::UpdateTheme(_) => write!(f, "UpdateTheme"),
            OrchestratorMessage::UpdateSystemTrayMode(_) => write!(f, "UpdateSystemTrayMode"),
            OrchestratorMessage::SaveSettings => write!(f, "SaveSettings"),
            OrchestratorMessage::RestartApp => write!(f, "RestartApp"),
            OrchestratorMessage::TrayEvent(event) => write!(f, "TrayEvent({:?})", event),
            OrchestratorMessage::HideMainWindow => write!(f, "HideMainWindow"),
            OrchestratorMessage::OpenOnboarding => write!(f, "OpenOnboarding"),
            OrchestratorMessage::OnboardingMsg(id, _) => write!(f, "OnboardingMsg({:?})", id),
            OrchestratorMessage::EnableKeyboardListener => write!(f, "EnableKeyboardListener"),
            OrchestratorMessage::CopyImageToClipboard(id, _, _) => {
                write!(f, "CopyImageToClipboard({:?})", id)
            }
            OrchestratorMessage::SaveImageToFile(id, _, _) => {
                write!(f, "SaveImageToFile({:?})", id)
            }
            OrchestratorMessage::OpenWindowPicker => write!(f, "OpenWindowPicker"),
            OrchestratorMessage::WindowPickerMsg(id, _) => {
                write!(f, "WindowPickerMsg({:?})", id)
            }
            OrchestratorMessage::WindowsListLoaded(id, windows) => {
                write!(f, "WindowsListLoaded({:?}, {} windows)", id, windows.len())
            }
            OrchestratorMessage::FocusWindowAndCapture(app_name) => {
                write!(f, "FocusWindowAndCapture({})", app_name)
            }
            OrchestratorMessage::CaptureSelectedWindow(window_id) => {
                write!(f, "CaptureSelectedWindow({})", window_id)
            }
            OrchestratorMessage::WindowCaptureComplete(_) => {
                write!(f, "WindowCaptureComplete")
            }
            OrchestratorMessage::WindowCaptureError(e) => {
                write!(f, "WindowCaptureError({})", e)
            }
        }
    }
}

impl AppOrchestrator {
    pub fn build(
        screen_capturer: Arc<dyn ScreenCapturer>,
        mouse_provider: Arc<dyn MousePositionProvider>,
        ocr_service: Arc<dyn OcrService>,
        reverse_image_search_provider: Arc<dyn ReverseImageSearchProvider>,
        settings: UserSettings,
    ) -> Self {
        Self {
            screen_capturer,
            mouse_provider,
            ocr_service,
            reverse_image_search_provider,
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

    fn refresh_correlation_id(&mut self) {
        let now = std::time::SystemTime::now();
        let elapsed = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        self.current_correlation_id = format!("{}{}", CORRELATION_ID_ORCHESTRATOR_PREFIX, elapsed);
    }

    pub(super) fn current_correlation_id(&self) -> String {
        self.current_correlation_id.clone()
    }

    pub(super) fn begin_settings_edit(&mut self) {
        self.settings_edit_state = SettingsEditState::Editing(self.settings.clone());
    }

    pub(super) fn discard_settings_edit(&mut self) {
        self.settings_edit_state = SettingsEditState::Closed;
    }

    pub(super) fn get_settings_for_rendering(&self) -> &UserSettings {
        match &self.settings_edit_state {
            SettingsEditState::Closed => &self.settings,
            SettingsEditState::Editing(settings) => settings,
        }
    }

    pub(super) fn update_settings_draft(
        &mut self,
        update_function: impl FnOnce(&mut UserSettings),
    ) -> bool {
        match &mut self.settings_edit_state {
            SettingsEditState::Closed => false,
            SettingsEditState::Editing(settings) => {
                update_function(settings);
                true
            }
        }
    }

    pub(super) fn take_settings_draft(&mut self) -> Option<UserSettings> {
        match std::mem::replace(&mut self.settings_edit_state, SettingsEditState::Closed) {
            SettingsEditState::Closed => None,
            SettingsEditState::Editing(settings) => Some(settings),
        }
    }

    pub(super) fn log_info_event(&self, event: &str, details: serde_json::Value) {
        log::info!(
            "{}",
            serde_json::json!({
                "event": event,
                "correlation_id": self.current_correlation_id,
                "details": details,
            })
        );
    }

    pub(super) fn log_error_event(&self, event: &str, details: serde_json::Value) {
        log::error!(
            "{}",
            serde_json::json!({
                "event": event,
                "correlation_id": self.current_correlation_id,
                "details": details,
            })
        );
    }

    pub(super) fn log_info_event_for_correlation(
        correlation_id: String,
        event: &str,
        details: serde_json::Value,
    ) {
        log::info!(
            "{}",
            serde_json::json!({
                "event": event,
                "correlation_id": correlation_id,
                "details": details,
            })
        );
    }

    pub(super) fn log_error_event_for_correlation(
        correlation_id: String,
        event: &str,
        details: serde_json::Value,
    ) {
        log::error!(
            "{}",
            serde_json::json!({
                "event": event,
                "correlation_id": correlation_id,
                "details": details,
            })
        );
    }

    pub(super) fn log_warn_event(&self, event: &str, details: serde_json::Value) {
        log::warn!(
            "{}",
            serde_json::json!({
                "event": event,
                "correlation_id": self.current_correlation_id,
                "details": details,
            })
        );
    }

    pub(super) fn log_debug_event(&self, event: &str, details: serde_json::Value) {
        log::debug!(
            "{}",
            serde_json::json!({
                "event": event,
                "correlation_id": self.current_correlation_id,
                "details": details,
            })
        );
    }

    #[allow(dead_code)]
    pub fn is_any_window_searching(&self) -> bool {
        for window in self.windows.values() {
            if matches!(window, AppWindow::InteractiveOcr(view) if view.is_searching()) {
                return true;
            }
        }
        false
    }

    pub fn create_hidden_window(&mut self) -> Task<OrchestratorMessage> {
        if self.hidden_window_id.is_some() {
            return Task::none();
        }

        log::info!("[ORCHESTRATOR] Creating hidden background window to keep app alive");

        let (id, task) = window::open(window::Settings {
            size: Size::new(1.0, 1.0),
            position: window::Position::Specific(Point::new(-10000.0, -10000.0)),
            visible: false,
            resizable: false,
            decorations: false,
            ..Default::default()
        });

        self.hidden_window_id = Some(id);
        self.windows.insert(id, AppWindow::Hidden);

        task.discard()
    }

    #[allow(dead_code)]
    pub fn get_window_title(&self, _window: Id) -> String {
        global_constants::APPLICATION_TITLE.to_string()
    }

    pub fn update(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        self.refresh_correlation_id();
        self.log_info_event(
            "orchestrator_message_received",
            serde_json::json!({
                "message": format!("{:?}", message),
            }),
        );

        match message {
            OrchestratorMessage::OpenMainWindow => {
                return self.handle_open_main_window();
            }
            OrchestratorMessage::CreateHiddenWindow => {
                return self.create_hidden_window();
            }
            OrchestratorMessage::CaptureScreen => {
                return self.handle_capture_screen();
            }
            OrchestratorMessage::PerformCapture => {
                return self.handle_perform_capture();
            }
            OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer) => {
                return self.handle_open_capture_overlay(mouse_x, mouse_y, capture_buffer);
            }
            OrchestratorMessage::CaptureError(error_msg) => {
                return self.handle_capture_error(error_msg);
            }
            OrchestratorMessage::Keyboard(GlobalKeyboardEvent::CaptureHotkeyPressed) => {
                log::info!("[ORCHESTRATOR] Capture hotkey pressed (Alt+Shift+S)");
                return self.update(OrchestratorMessage::CaptureScreen);
            }
            OrchestratorMessage::Keyboard(GlobalKeyboardEvent::EscapePressed) => {
                return self.handle_escape_pressed();
            }
            OrchestratorMessage::CaptureOverlayMessage(window_id, capture_msg) => {
                return self.handle_capture_overlay_message(window_id, capture_msg);
            }
            OrchestratorMessage::ConfirmSelection(overlay_id) => {
                return self.handle_confirm_selection(overlay_id);
            }
            OrchestratorMessage::ShowCroppedImage(capture_buffer, selection_rect) => {
                return self.handle_show_cropped_image(capture_buffer, selection_rect);
            }
            OrchestratorMessage::ProcessOcr(window_id, buffer) => {
                return self.handle_process_ocr(window_id, buffer);
            }
            OrchestratorMessage::OcrComplete(window_id, result) => {
                return self.handle_ocr_complete(window_id, result);
            }
            OrchestratorMessage::OcrServiceReady(service) => {
                return self.handle_ocr_service_ready(service);
            }
            OrchestratorMessage::OcrServiceFailed(error) => {
                return self.handle_ocr_service_failed(error);
            }
            OrchestratorMessage::InteractiveOcrMessage(window_id, ocr_msg) => {
                return self.handle_interactive_ocr_message(window_id, ocr_msg);
            }
            OrchestratorMessage::PerformImageSearch(window_id, buffer, query) => {
                return self.handle_perform_image_search(window_id, buffer, query);
            }
            OrchestratorMessage::SpinnerTick => {
                for (_window_id, window) in &mut self.windows {
                    if let AppWindow::InteractiveOcr(view) = window {
                        view.update(crate::presentation::InteractiveOcrMessage::SpinnerTick);
                    }
                    if let AppWindow::WindowPicker(view) = window {
                        view.update(crate::presentation::WindowPickerMessage::SpinnerTick);
                    }
                }
            }
            OrchestratorMessage::CloseWindow(id) => {
                log::info!("[ORCHESTRATOR] Closing window: {:?}", id);
                return window::close(id);
            }
            OrchestratorMessage::WindowClosed(id) => {
                return self.handle_window_closed(id);
            }
            OrchestratorMessage::WindowFocused(id) => {
                return self.handle_window_focused(id);
            }
            OrchestratorMessage::OpenSettings => {
                return self.handle_open_settings();
            }
            OrchestratorMessage::UpdateSearchUrl(url) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_search_url_template = url;
                });
            }
            OrchestratorMessage::UpdateImageHostingProviderUrl(provider_url) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_provider_url = provider_url;
                });
            }
            OrchestratorMessage::UpdateImageHostingAuthMode(auth_mode) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_auth_mode = auth_mode;
                });
            }
            OrchestratorMessage::UpdateImageHostingPublicKeyName(public_key_name) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_public_key_name = public_key_name;
                });
            }
            OrchestratorMessage::UpdateImageHostingPublicKeyValue(public_key_value) => {
                if public_key_value.trim().is_empty() {
                    return Task::none();
                }
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_public_key_value = public_key_value;
                });
            }
            OrchestratorMessage::UpdateImageHostingExpirationSeconds(expiration_seconds) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_expiration_seconds = expiration_seconds;
                });
            }
            OrchestratorMessage::UpdateImageHostingHttpMethod(http_method) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_http_method = http_method;
                });
            }
            OrchestratorMessage::UpdateImageHostingImageFieldName(image_field_name) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.image_hosting_image_field_name = image_field_name;
                });
            }
            OrchestratorMessage::UpdateHotkey(hotkey) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.capture_hotkey = hotkey;
                });
            }
            OrchestratorMessage::UpdateTheme(theme) => {
                let _ = self.update_settings_draft(|settings| {
                    settings.theme_mode = theme;
                });
            }
            OrchestratorMessage::UpdateSystemTrayMode(enabled) => {
                self.settings.run_in_system_tray = enabled;
                if let Err(save_error) = self.settings.save() {
                    self.log_error_event(
                        "system_tray_setting_save_failed",
                        serde_json::json!({"error": save_error.to_string()}),
                    );
                }
                if enabled {
                    return self.handle_hide_main_window();
                } else {
                    return self.handle_open_main_window();
                }
            }
            OrchestratorMessage::SaveSettings => {
                return self.handle_save_settings();
            }
            OrchestratorMessage::RestartApp => {
                return self.handle_restart_app();
            }
            OrchestratorMessage::TrayEvent(event) => {
                return self.handle_tray_event(event);
            }
            OrchestratorMessage::HideMainWindow => {
                return self.handle_hide_main_window();
            }
            OrchestratorMessage::OpenOnboarding => {
                return self.handle_open_onboarding();
            }
            OrchestratorMessage::OnboardingMsg(window_id, msg) => {
                return self.handle_onboarding_message(window_id, msg);
            }
            OrchestratorMessage::EnableKeyboardListener => {
                log::debug!("[ORCHESTRATOR] EnableKeyboardListener handled at app level");
            }
            OrchestratorMessage::CopyImageToClipboard(window_id, buffer, draw_strokes) => {
                return self.handle_copy_image_to_clipboard(window_id, buffer, draw_strokes);
            }
            OrchestratorMessage::SaveImageToFile(window_id, buffer, draw_strokes) => {
                return self.handle_save_image_to_file(window_id, buffer, draw_strokes);
            }
            OrchestratorMessage::OpenWindowPicker => {
                return self.handle_open_window_picker();
            }
            OrchestratorMessage::WindowPickerMsg(window_id, msg) => {
                return self.handle_window_picker_message(window_id, msg);
            }
            OrchestratorMessage::WindowsListLoaded(window_id, windows) => {
                return self.handle_windows_list_loaded(window_id, windows);
            }
            OrchestratorMessage::FocusWindowAndCapture(app_name) => {
                return self.handle_focus_window_and_capture(app_name);
            }
            OrchestratorMessage::CaptureSelectedWindow(window_id) => {
                return self.handle_capture_selected_window(window_id);
            }
            OrchestratorMessage::WindowCaptureComplete(capture_buffer) => {
                return self.handle_window_capture_complete(capture_buffer);
            }
            OrchestratorMessage::WindowCaptureError(error_msg) => {
                self.log_error_event(
                    "window_capture_failed",
                    serde_json::json!({
                        "error": error_msg,
                    }),
                );
            }
        }

        self.log_info_event(
            "orchestrator_message_completed",
            serde_json::json!({
                "result": "no_task",
            }),
        );
        Task::none()
    }

    pub fn render_view(&self, window_id: Id) -> Element<'_, OrchestratorMessage> {
        match self.windows.get(&window_id) {
            Some(AppWindow::Main) => self.render_main_window(),
            Some(AppWindow::CaptureOverlay(capture_view)) => capture_view
                .render_ui()
                .map(move |msg| OrchestratorMessage::CaptureOverlayMessage(window_id, msg)),
            Some(AppWindow::InteractiveOcr(ocr_view)) => ocr_view
                .render_ui()
                .map(move |msg| OrchestratorMessage::InteractiveOcrMessage(window_id, msg)),
            Some(AppWindow::Settings) => self.render_settings_window(),
            Some(AppWindow::Onboarding(onboarding_view)) => onboarding_view
                .view()
                .map(move |msg| OrchestratorMessage::OnboardingMsg(window_id, msg)),
            Some(AppWindow::Hidden) => container(Space::new()).into(),
            Some(AppWindow::WindowPicker(picker_view)) => picker_view
                .render_ui()
                .map(move |msg| OrchestratorMessage::WindowPickerMsg(window_id, msg)),
            None => text(global_constants::UI_GENERIC_LOADING).into(),
        }
    }
}

#[cfg(test)]
mod tests;
