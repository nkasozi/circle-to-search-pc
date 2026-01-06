use std::collections::HashMap;
use std::sync::Arc;

use iced::widget::{button, column, container, row, text, Space};
use iced::window::{self, Id};
use iced::{Alignment, Background, Color, Element, Length, Point, Rectangle, Size, Task};
use mouse_position::mouse_position::Mouse;

use crate::adapters::{auto_launch, macos_permissions};
use crate::core::interfaces::adapters::{OcrService, ReverseImageSearchProvider};
use crate::core::interfaces::ports::{MousePositionProvider, ScreenCapturer};
use crate::core::models::{CaptureBuffer, OcrResult, ScreenRegion, ThemeMode, UserSettings};
use crate::ports::{GlobalKeyboardEvent, TrayEvent};
use crate::presentation::app_theme;
use crate::presentation::{CaptureView, CaptureViewMessage, OnboardingMessage, OnboardingView};

pub enum AppWindow {
    Main,
    CaptureOverlay(CaptureView),
    InteractiveOcr(crate::presentation::InteractiveOcrView),
    Settings,
    Onboarding(OnboardingView),
    Hidden,
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
    status: String,
    settings: UserSettings,
    settings_window_id: Option<Id>,
    temp_settings: Option<UserSettings>,
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
    PerformImageSearch(Id, CaptureBuffer),
    #[allow(dead_code)]
    CloseWindow(Id),
    WindowClosed(Id),
    Keyboard(GlobalKeyboardEvent),
    OpenSettings,
    UpdateSearchUrl(String),
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
            OrchestratorMessage::PerformImageSearch(id, _) => {
                write!(f, "PerformImageSearch({:?})", id)
            }
            OrchestratorMessage::CloseWindow(id) => write!(f, "CloseWindow({:?})", id),
            OrchestratorMessage::WindowClosed(id) => write!(f, "WindowClosed({:?})", id),
            OrchestratorMessage::Keyboard(event) => write!(f, "Keyboard({:?})", event),
            OrchestratorMessage::OpenSettings => write!(f, "OpenSettings"),
            OrchestratorMessage::UpdateSearchUrl(_) => write!(f, "UpdateSearchUrl"),
            OrchestratorMessage::UpdateHotkey(_) => write!(f, "UpdateHotkey"),
            OrchestratorMessage::UpdateTheme(_) => write!(f, "UpdateTheme"),
            OrchestratorMessage::UpdateSystemTrayMode(_) => write!(f, "UpdateSystemTrayMode"),
            OrchestratorMessage::SaveSettings => write!(f, "SaveSettings"),
            OrchestratorMessage::RestartApp => write!(f, "RestartApp"),
            OrchestratorMessage::TrayEvent(event) => write!(f, "TrayEvent({:?})", event),
            OrchestratorMessage::HideMainWindow => write!(f, "HideMainWindow"),
            OrchestratorMessage::OpenOnboarding => write!(f, "OpenOnboarding"),
            OrchestratorMessage::OnboardingMsg(id, _) => write!(f, "OnboardingMsg({:?})", id),
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
            status: "Initializing OCR service...".to_string(),
            settings,
            settings_window_id: None,
            temp_settings: None,
        }
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
        "Circle to Search".to_string()
    }

    pub fn update(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Received message: {:?}", message);

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
            OrchestratorMessage::PerformImageSearch(window_id, buffer) => {
                return self.handle_perform_image_search(window_id, buffer);
            }
            OrchestratorMessage::CloseWindow(id) => {
                log::info!("[ORCHESTRATOR] Closing window: {:?}", id);
                return window::close(id);
            }
            OrchestratorMessage::WindowClosed(id) => {
                return self.handle_window_closed(id);
            }
            OrchestratorMessage::OpenSettings => {
                return self.handle_open_settings();
            }
            OrchestratorMessage::UpdateSearchUrl(url) => {
                if let Some(ref mut temp) = self.temp_settings {
                    temp.image_search_url_template = url;
                }
            }
            OrchestratorMessage::UpdateHotkey(hotkey) => {
                if let Some(ref mut temp) = self.temp_settings {
                    temp.capture_hotkey = hotkey;
                }
            }
            OrchestratorMessage::UpdateTheme(theme) => {
                if let Some(ref mut temp) = self.temp_settings {
                    temp.theme_mode = theme;
                }
            }
            OrchestratorMessage::UpdateSystemTrayMode(enabled) => {
                self.settings.run_in_system_tray = enabled;
                if let Err(e) = self.settings.save() {
                    log::error!("[ORCHESTRATOR] Failed to save system tray setting: {}", e);
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
        }

        log::debug!("[ORCHESTRATOR] No task to return, ending update");
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
            None => text("Loading...").into(),
        }
    }

    fn handle_open_main_window(&mut self) -> Task<OrchestratorMessage> {
        log::debug!("[ORCHESTRATOR] Opening main window");

        if self.main_window_id.is_some() && self.windows.contains_key(&self.main_window_id.unwrap())
        {
            log::warn!("[ORCHESTRATOR] Main window already exists and is open");
            return Task::none();
        }

        let (id, task) = window::open(window::Settings {
            size: Size::new(700.0, 650.0),
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });

        self.main_window_id = Some(id);
        self.windows.insert(id, AppWindow::Main);
        log::info!("[ORCHESTRATOR] Main window created with ID: {:?}", id);
        task.discard()
    }

    fn handle_capture_screen(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Starting capture screen process");
        self.status = "Preparing to capture...".to_string();

        let main_window_id = self.main_window_id;

        log::debug!("[ORCHESTRATOR] Minimizing main window and waiting 200ms before capture");
        Task::batch(vec![
            if let Some(id) = main_window_id {
                window::minimize(id, true)
            } else {
                Task::none()
            },
            Task::future(async {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                log::debug!("[ORCHESTRATOR] Delay complete, triggering PerformCapture");
                OrchestratorMessage::PerformCapture
            }),
        ])
    }

    fn handle_perform_capture(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Performing screen capture");
        self.status = "Capturing screen...".to_string();

        let screen_capturer = Arc::clone(&self.screen_capturer);

        Task::future(async move {
            log::debug!("[ORCHESTRATOR] Getting mouse position");
            let (mouse_x, mouse_y) = match Mouse::get_mouse_position() {
                Mouse::Position { x, y } => {
                    log::debug!("[ORCHESTRATOR] Mouse position: ({}, {})", x, y);
                    (x, y)
                }
                Mouse::Error => {
                    log::warn!("[ORCHESTRATOR] Failed to get mouse position, using (0,0)");
                    (0, 0)
                }
            };

            let region = ScreenRegion::at_coordinates(mouse_x, mouse_y);
            log::debug!("[ORCHESTRATOR] Capturing screen at region");

            match screen_capturer.capture_screen_at_region(&region) {
                Ok(capture_buffer) => {
                    log::info!(
                        "[ORCHESTRATOR] Screen captured successfully, buffer size: {}x{}",
                        capture_buffer.width,
                        capture_buffer.height
                    );
                    OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer)
                }
                Err(e) => {
                    log::error!("[ORCHESTRATOR] Screen capture failed: {}. If multiple instances are running, this may be expected.", e);
                    OrchestratorMessage::CaptureError(format!(
                        "Capture failed: {}. Try closing other instances.",
                        e
                    ))
                }
            }
        })
    }

    fn handle_open_capture_overlay(
        &mut self,
        mouse_x: i32,
        mouse_y: i32,
        capture_buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        log::info!(
            "[ORCHESTRATOR] Opening capture overlay at ({}, {})",
            mouse_x,
            mouse_y
        );
        match xcap::Monitor::from_point(mouse_x, mouse_y) {
            Ok(monitor) => {
                log::debug!("[ORCHESTRATOR] Monitor found, creating overlay window");
                let (id, task) = window::open(window::Settings {
                    position: window::Position::Specific(Point::new(
                        monitor.x().unwrap_or(0) as f32,
                        monitor.y().unwrap_or(0) as f32,
                    )),
                    size: Size::new(
                        monitor.width().unwrap_or(1920) as f32,
                        monitor.height().unwrap_or(1080) as f32,
                    ),
                    transparent: true,
                    decorations: false,
                    ..Default::default()
                });

                let capture_view = CaptureView::build_with_capture_buffer(capture_buffer);
                self.windows
                    .insert(id, AppWindow::CaptureOverlay(capture_view));
                self.status = "Overlay ready!".to_string();
                log::info!("[ORCHESTRATOR] Overlay window created with ID: {:?}", id);

                return task.discard().chain(window::gain_focus(id));
            }
            Err(e) => {
                log::error!("[ORCHESTRATOR] Failed to get monitor: {}", e);
                self.status = format!("Monitor error: {}", e);
            }
        }
        Task::none()
    }

    fn handle_capture_error(&mut self, error_msg: String) -> Task<OrchestratorMessage> {
        log::error!("[ORCHESTRATOR] Capture error: {}", error_msg);
        self.status = error_msg;
        Task::none()
    }

    fn handle_escape_pressed(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Escape key pressed");
        if let Some((id, AppWindow::CaptureOverlay(_))) = self
            .windows
            .iter()
            .find(|(_, w)| matches!(w, AppWindow::CaptureOverlay(_)))
        {
            log::debug!("[ORCHESTRATOR] Closing overlay window: {:?}", id);
            return window::close(*id);
        }
        log::debug!("[ORCHESTRATOR] No overlay window found to close");
        self.status = "Ready - Press Alt+Shift+S to capture".to_string();
        Task::none()
    }

    fn handle_capture_overlay_message(
        &mut self,
        window_id: Id,
        capture_msg: CaptureViewMessage,
    ) -> Task<OrchestratorMessage> {
        log::debug!(
            "[ORCHESTRATOR] Received overlay message for window {:?}: {:?}",
            window_id,
            capture_msg
        );
        if let CaptureViewMessage::ConfirmSelection = capture_msg {
            log::info!("[ORCHESTRATOR] Selection confirmed by overlay");
            return self.update(OrchestratorMessage::ConfirmSelection(window_id));
        }
        if let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get_mut(&window_id) {
            log::debug!("[ORCHESTRATOR] Updating overlay view state");
            capture_view.update(capture_msg);
        } else {
            log::warn!("[ORCHESTRATOR] Overlay window {:?} not found", window_id);
        }
        Task::none()
    }

    fn handle_confirm_selection(&mut self, overlay_id: Id) -> Task<OrchestratorMessage> {
        log::info!(
            "[ORCHESTRATOR] Confirming selection from overlay {:?}",
            overlay_id
        );

        if let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get(&overlay_id) {
            if let Some(selection_rect) = capture_view.get_selected_region() {
                log::info!("[ORCHESTRATOR] Selection region: {:?}", selection_rect);
                let capture_buffer = capture_view.get_capture_buffer().clone();

                self.status = "Processing selection...".to_string();
                return Task::batch(vec![
                    window::close(overlay_id),
                    Task::done(OrchestratorMessage::ShowCroppedImage(
                        capture_buffer,
                        selection_rect,
                    )),
                ]);
            }
            log::warn!("[ORCHESTRATOR] No selection region found");
        } else {
            log::warn!("[ORCHESTRATOR] Overlay window not found");
        }

        window::close(overlay_id)
    }

    fn handle_show_cropped_image(
        &mut self,
        capture_buffer: CaptureBuffer,
        selection_rect: Rectangle,
    ) -> Task<OrchestratorMessage> {
        log::info!(
            "[ORCHESTRATOR] Showing cropped image from selection: {:?}",
            selection_rect
        );

        let cropped_buffer = capture_buffer.crop_region(
            selection_rect.x as u32,
            selection_rect.y as u32,
            selection_rect.width as u32,
            selection_rect.height as u32,
        );

        match cropped_buffer {
            Ok(buffer) => {
                log::info!(
                    "[ORCHESTRATOR] Successfully cropped image: {}x{}",
                    buffer.width,
                    buffer.height
                );

                let (id, task) = window::open(window::Settings {
                    size: Size::new(
                        (buffer.width as f32).min(1200.0),
                        (buffer.height as f32).min(800.0),
                    ),
                    position: window::Position::Centered,
                    resizable: true,
                    ..Default::default()
                });

                let view = crate::presentation::InteractiveOcrView::build(
                    buffer.clone(),
                    self.settings.theme_mode.clone(),
                );
                self.windows.insert(id, AppWindow::InteractiveOcr(view));
                self.status = "Processing OCR...".to_string();

                return Task::batch(vec![
                    task.discard(),
                    Task::done(OrchestratorMessage::ProcessOcr(id, buffer)),
                ]);
            }
            Err(e) => {
                log::error!("[ORCHESTRATOR] Failed to crop image: {}", e);
                self.status = format!("Error cropping image: {}", e);
            }
        }
        Task::none()
    }

    fn handle_process_ocr(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        log::info!(
            "[ORCHESTRATOR] Starting OCR processing for window {:?}",
            window_id
        );

        let ocr_service = self.ocr_service.clone();
        let width = buffer.width;
        let height = buffer.height;

        Task::future(async move {
            log::debug!(
                "[OCR] Converting capture buffer to dynamic image {}x{}",
                width,
                height
            );

            let dynamic_image = match image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, buffer.raw_data.clone())
                    .expect("Failed to create image from raw data"),
            ) {
                img => img,
            };

            log::debug!("[OCR] Running OCR on image");
            match ocr_service.extract_text_from_image(&dynamic_image).await {
                Ok(result) => {
                    log::info!(
                        "[OCR] OCR completed successfully. Found {} text blocks",
                        result.text_blocks.len()
                    );
                    OrchestratorMessage::OcrComplete(window_id, Ok(result))
                }
                Err(e) => {
                    log::error!("[OCR] OCR failed: {}", e);
                    OrchestratorMessage::OcrComplete(window_id, Err(e.to_string()))
                }
            }
        })
    }

    fn handle_ocr_complete(
        &mut self,
        window_id: Id,
        result: Result<OcrResult, String>,
    ) -> Task<OrchestratorMessage> {
        match result {
            Ok(ocr_result) => {
                log::info!(
                    "[ORCHESTRATOR] OCR complete for window {:?}: {} text blocks found",
                    window_id,
                    ocr_result.text_blocks.len()
                );

                if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
                    view.set_ocr_result(ocr_result);
                    self.status = "OCR complete".to_string();
                }
            }
            Err(e) => {
                log::error!(
                    "[ORCHESTRATOR] OCR failed for window {:?}: {}",
                    window_id,
                    e
                );
                self.status = format!("OCR failed: {}", e);
            }
        }
        Task::none()
    }

    fn handle_ocr_service_ready(
        &mut self,
        service: Arc<dyn OcrService>,
    ) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] OCR service is ready");
        self.ocr_service = service;
        self.status = "Ready - Press Alt+Shift+S to capture".to_string();
        Task::none()
    }

    fn handle_ocr_service_failed(&mut self, error: String) -> Task<OrchestratorMessage> {
        log::error!(
            "[ORCHESTRATOR] OCR service initialization failed: {}",
            error
        );
        self.status = format!("OCR initialization failed: {}", error);
        Task::none()
    }

    fn handle_interactive_ocr_message(
        &mut self,
        window_id: Id,
        ocr_msg: crate::presentation::InteractiveOcrMessage,
    ) -> Task<OrchestratorMessage> {
        log::debug!(
            "[ORCHESTRATOR] Received OCR message for window {:?}: {:?}",
            window_id,
            ocr_msg
        );

        if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
            view.update(ocr_msg.clone());
        }

        match ocr_msg {
            crate::presentation::InteractiveOcrMessage::Close => {
                return window::close(window_id);
            }
            crate::presentation::InteractiveOcrMessage::SearchSelected => {
                if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) {
                    let buffer = view.get_capture_buffer().clone();
                    return Task::done(OrchestratorMessage::PerformImageSearch(window_id, buffer));
                }
            }
            crate::presentation::InteractiveOcrMessage::CopySelected => {
                return Task::future(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    OrchestratorMessage::InteractiveOcrMessage(
                        window_id,
                        crate::presentation::InteractiveOcrMessage::HideToast,
                    )
                });
            }
            _ => {}
        }
        Task::none()
    }

    fn handle_perform_image_search(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
    ) -> Task<OrchestratorMessage> {
        log::info!(
            "[ORCHESTRATOR] Starting image search for window {:?}",
            window_id
        );

        let search_provider = Arc::clone(&self.reverse_image_search_provider);

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                crate::presentation::InteractiveOcrMessage::SearchUploading,
            )),
            Task::future(async move {
                let search_future = search_provider.perform_search(&buffer);

                let timeout_duration = std::time::Duration::from_secs(30);
                match tokio::time::timeout(timeout_duration, search_future).await {
                    Ok(Ok(_search_url)) => {
                        log::info!("[ORCHESTRATOR] Image search completed successfully");
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchCompleted,
                        )
                    }
                    Ok(Err(e)) => {
                        log::error!("[ORCHESTRATOR] Image search failed: {}", e);
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchFailed(e.to_string()),
                        )
                    }
                    Err(_) => {
                        log::error!("[ORCHESTRATOR] Image search timed out after 30 seconds");
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchFailed(
                                "Search timed out after 30 seconds".to_string(),
                            ),
                        )
                    }
                }
            }),
        ])
    }

    fn handle_window_closed(&mut self, id: Id) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Window closed: {:?}", id);

        if Some(id) == self.hidden_window_id {
            log::warn!("[ORCHESTRATOR] Hidden window closed unexpectedly, recreating");
            self.hidden_window_id = None;
            self.windows.remove(&id);
            return self.create_hidden_window();
        }

        if Some(id) == self.main_window_id {
            log::info!("[ORCHESTRATOR] Main window closed, app will continue in system tray");
            self.windows.remove(&id);
            self.main_window_id = None;
            return Task::none();
        }

        if Some(id) == self.onboarding_window_id {
            log::info!("[ORCHESTRATOR] Onboarding window closed");
            self.windows.remove(&id);
            self.onboarding_window_id = None;
            return Task::none();
        }

        let was_ocr_window = matches!(self.windows.get(&id), Some(AppWindow::InteractiveOcr(_)));
        self.windows.remove(&id);
        if Some(id) == self.settings_window_id {
            self.settings_window_id = None;
            self.temp_settings = None;
        }
        log::debug!(
            "[ORCHESTRATOR] Removed window from tracking. Remaining: {}",
            self.windows.len()
        );
        self.status = "Ready - Press Alt+Shift+S to capture".to_string();

        if was_ocr_window {
            if let Some(main_id) = self.main_window_id {
                return window::minimize(main_id, false);
            }
        }
        Task::none()
    }

    fn handle_open_settings(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Opening settings window");
        if self.settings_window_id.is_some() {
            log::warn!("[ORCHESTRATOR] Settings window already open");
            return Task::none();
        }

        let (id, task) = window::open(window::Settings {
            size: Size::new(500.0, 550.0),
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });

        self.settings_window_id = Some(id);
        self.temp_settings = Some(self.settings.clone());
        self.windows.insert(id, AppWindow::Settings);
        log::info!("[ORCHESTRATOR] Settings window created with ID: {:?}", id);

        task.discard()
    }

    fn handle_save_settings(&mut self) -> Task<OrchestratorMessage> {
        if let Some(temp) = self.temp_settings.take() {
            let hotkey_changed = temp.capture_hotkey != self.settings.capture_hotkey;

            self.settings = temp.clone();
            if let Err(e) = self.settings.save() {
                log::error!("[ORCHESTRATOR] Failed to save settings: {}", e);
                self.status = format!("Failed to save settings: {}", e);
            } else {
                log::info!("[ORCHESTRATOR] Settings saved successfully");
                self.status = "Settings saved".to_string();

                if hotkey_changed {
                    log::info!("[ORCHESTRATOR] Hotkey changed, restarting app...");
                    return Task::done(OrchestratorMessage::RestartApp);
                }
            }
        }

        if let Some(id) = self.settings_window_id {
            return window::close(id);
        }
        Task::none()
    }

    fn handle_restart_app(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Restarting application...");
        let exe_path = std::env::current_exe().expect("Failed to get executable path");
        std::process::Command::new(exe_path)
            .spawn()
            .expect("Failed to restart app");
        std::process::exit(0);
    }

    fn handle_tray_event(&mut self, event: TrayEvent) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Handling tray event: {:?}", event);

        match event {
            TrayEvent::ShowWindow => self.handle_open_main_window(),
            TrayEvent::OpenSettings => self.handle_open_settings(),
            TrayEvent::Quit => {
                log::info!("[ORCHESTRATOR] Quit requested from tray");
                iced::exit()
            }
        }
    }

    fn handle_hide_main_window(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Hiding main window to system tray");
        if let Some(id) = self.main_window_id {
            window::close(id)
        } else {
            Task::none()
        }
    }

    fn render_main_window(&self) -> Element<'_, OrchestratorMessage> {
        let theme = app_theme::get_theme(&self.settings.theme_mode);

        let logo_icon = text("🔍").size(64);

        let title = text("Circle to Search").size(36);

        let subtitle = text("Search anything on your screen instantly")
            .size(16)
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
            });

        let header_section = column![logo_icon, title, subtitle]
            .spacing(8)
            .align_x(Alignment::Center);

        let capture_btn = button(
            row![text("📸").size(24), text("Capture Screen").size(18)]
                .spacing(12)
                .align_y(Alignment::Center),
        )
        .padding([16, 48])
        .style(|theme, status| app_theme::primary_button_style(theme, status))
        .on_press(OrchestratorMessage::CaptureScreen);

        let hotkey_hint = container(
            text(format!(
                "or press {} anywhere",
                &self.settings.capture_hotkey
            ))
            .size(13)
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
            }),
        );

        let action_section = column![capture_btn, hotkey_hint]
            .spacing(12)
            .align_x(Alignment::Center);

        let status_indicator = self.render_status_indicator();

        let divider = container(text(""))
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(1.0))
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.3))),
                ..Default::default()
            });

        let system_tray_row = row![
            iced::widget::checkbox(self.settings.run_in_system_tray)
                .on_toggle(OrchestratorMessage::UpdateSystemTrayMode),
            text("Keep running in background").size(14),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let settings_btn = button(
            row![text("⚙").size(16), text("Settings").size(14)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding([10, 24])
        .style(|theme, status| app_theme::secondary_button_style(theme, status))
        .on_press(OrchestratorMessage::OpenSettings);

        let footer_section = column![system_tray_row, settings_btn]
            .spacing(16)
            .align_x(Alignment::Center);

        let content = column![
            header_section,
            Space::new().height(Length::Fixed(32.0)),
            action_section,
            Space::new().height(Length::Fixed(16.0)),
            status_indicator,
            divider,
            footer_section,
        ]
        .spacing(4)
        .padding(40)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(Background::Color(palette.background)),
                    text_color: Some(palette.text),
                    ..Default::default()
                }
            })
            .into()
    }

    fn render_status_indicator(&self) -> Element<'_, OrchestratorMessage> {
        let (status_color, status_icon) = match self.status.as_str() {
            s if s.contains("Ready") => (Color::from_rgb(0.2, 0.8, 0.4), "●"),
            s if s.contains("Loading") || s.contains("Initializing") => {
                (Color::from_rgb(1.0, 0.8, 0.2), "○")
            }
            s if s.contains("Error") || s.contains("Failed") => {
                (Color::from_rgb(1.0, 0.3, 0.3), "●")
            }
            _ => (Color::from_rgba(0.5, 0.5, 0.5, 1.0), "●"),
        };

        let status_text = row![
            text(status_icon)
                .size(12)
                .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(status_color),
                }),
            text(&self.status)
                .size(13)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        container(status_text).into()
    }

    fn render_settings_window(&self) -> Element<'_, OrchestratorMessage> {
        use iced::widget::{pick_list, text_input};

        let theme = app_theme::get_theme(&self.settings.theme_mode);
        let temp = self.temp_settings.as_ref().unwrap_or(&self.settings);

        let header_icon = text("⚙").size(48);
        let title = text("Settings").size(28);
        let header_section = column![header_icon, title]
            .spacing(8)
            .align_x(Alignment::Center);

        let search_section = self.render_settings_section(
            "Search",
            "🔍",
            column![self.render_setting_row(
                "Image Search URL",
                "Template URL for reverse image search",
                text_input(
                    "https://lens.google.com/uploadbyurl?url={}",
                    &temp.image_search_url_template,
                )
                .on_input(OrchestratorMessage::UpdateSearchUrl)
                .padding(12)
                .into(),
            ),]
            .spacing(12),
        );

        let hotkey_warning =
            text("Requires app restart to take effect")
                .size(11)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 0.7, 0.0, 0.8)),
                });

        let keyboard_section = self.render_settings_section(
            "Keyboard",
            "⌨",
            column![self.render_setting_row(
                "Capture Hotkey",
                "Global shortcut to start capture",
                column![
                    text_input("Alt+Shift+S", &temp.capture_hotkey)
                        .on_input(OrchestratorMessage::UpdateHotkey)
                        .padding(12),
                    hotkey_warning,
                ]
                .spacing(4)
                .into(),
            ),]
            .spacing(12),
        );

        let appearance_section = self.render_settings_section(
            "Appearance",
            "🎨",
            column![self.render_setting_row(
                "Theme",
                "Choose light or dark mode",
                pick_list(
                    vec![ThemeMode::Dark, ThemeMode::Light,],
                    Some(temp.theme_mode.clone()),
                    OrchestratorMessage::UpdateTheme,
                )
                .padding(12)
                .into(),
            ),]
            .spacing(12),
        );

        let save_btn = button(
            row![text("💾").size(16), text("Save Changes").size(15)]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .padding([14, 36])
        .style(|theme, status| app_theme::primary_button_style(theme, status))
        .on_press(OrchestratorMessage::SaveSettings);

        let content = column![
            header_section,
            Space::new().height(Length::Fixed(24.0)),
            search_section,
            Space::new().height(Length::Fixed(16.0)),
            keyboard_section,
            Space::new().height(Length::Fixed(16.0)),
            appearance_section,
            Space::new().height(Length::Fixed(28.0)),
            save_btn,
        ]
        .spacing(4)
        .padding(32)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(Background::Color(palette.background)),
                    text_color: Some(palette.text),
                    ..Default::default()
                }
            })
            .into()
    }

    fn render_settings_section<'a>(
        &self,
        title: &'a str,
        icon: &'a str,
        content: iced::widget::Column<'a, OrchestratorMessage>,
    ) -> Element<'a, OrchestratorMessage> {
        let section_header = row![text(icon).size(18), text(title).size(16),]
            .spacing(8)
            .align_y(Alignment::Center);

        let section_content = container(content)
            .padding([12, 16])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: iced::Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });

        column![section_header, section_content]
            .spacing(8)
            .width(Length::Fill)
            .into()
    }

    fn render_setting_row<'a>(
        &self,
        label: &'a str,
        description: &'a str,
        input: Element<'a, OrchestratorMessage>,
    ) -> Element<'a, OrchestratorMessage> {
        let label_col = column![
            text(label).size(14),
            text(description)
                .size(11)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
        ]
        .spacing(2)
        .width(Length::FillPortion(2));

        let input_col = container(input).width(Length::FillPortion(3));

        row![label_col, input_col]
            .spacing(16)
            .align_y(Alignment::Center)
            .into()
    }

    fn handle_open_onboarding(&mut self) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Opening onboarding window");

        if self.onboarding_window_id.is_some()
            && self
                .windows
                .contains_key(&self.onboarding_window_id.unwrap())
        {
            log::warn!("[ORCHESTRATOR] Onboarding window already exists and is open");
            return Task::none();
        }

        let screen_recording_granted =
            macos_permissions::macos::check_screen_recording_permission();
        let accessibility_granted = macos_permissions::macos::check_accessibility_permission();
        let launch_at_login = auto_launch::is_launch_at_login_enabled();

        let onboarding_view = OnboardingView::new(
            screen_recording_granted,
            accessibility_granted,
            launch_at_login,
        );

        let (id, task) = window::open(window::Settings {
            size: Size::new(600.0, 700.0),
            position: window::Position::Centered,
            resizable: false,
            ..Default::default()
        });

        self.onboarding_window_id = Some(id);
        self.windows
            .insert(id, AppWindow::Onboarding(onboarding_view));
        log::info!("[ORCHESTRATOR] Onboarding window created with ID: {:?}", id);
        task.discard()
    }

    fn handle_onboarding_message(
        &mut self,
        window_id: Id,
        message: OnboardingMessage,
    ) -> Task<OrchestratorMessage> {
        log::debug!("[ORCHESTRATOR] Handling onboarding message: {:?}", message);

        match message {
            OnboardingMessage::OpenScreenRecordingSettings => {
                macos_permissions::macos::open_screen_recording_settings();
                return Task::none();
            }
            OnboardingMessage::OpenAccessibilitySettings => {
                macos_permissions::macos::open_accessibility_settings();
                return Task::none();
            }
            OnboardingMessage::RefreshPermissions => {
                let screen_recording_granted =
                    macos_permissions::macos::check_screen_recording_permission();
                let accessibility_granted =
                    macos_permissions::macos::check_accessibility_permission();

                if let Some(AppWindow::Onboarding(view)) = self.windows.get_mut(&window_id) {
                    view.update_permissions(screen_recording_granted, accessibility_granted);
                }
                return Task::none();
            }
            OnboardingMessage::FinishOnboarding => {
                return self.handle_finish_onboarding(window_id);
            }
            _ => {}
        }

        if let Some(AppWindow::Onboarding(view)) = self.windows.get_mut(&window_id) {
            view.handle_message(message);
        }

        Task::none()
    }

    fn handle_finish_onboarding(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Finishing onboarding");

        let launch_at_login =
            if let Some(AppWindow::Onboarding(view)) = self.windows.get(&window_id) {
                view.is_launch_at_login_enabled()
            } else {
                false
            };

        self.settings.onboarding_complete = true;
        self.settings.launch_at_login = launch_at_login;

        if let Err(error) = self.settings.save() {
            log::error!("[ORCHESTRATOR] Failed to save settings: {}", error);
        }

        auto_launch::set_launch_at_login(launch_at_login);

        self.windows.remove(&window_id);
        self.onboarding_window_id = None;

        let close_task = window::close(window_id);

        let open_main_task = if !self.settings.run_in_system_tray {
            Task::done(OrchestratorMessage::OpenMainWindow)
        } else {
            Task::none()
        };

        Task::batch(vec![close_task, open_main_task])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::OcrResult;

    struct MockScreenCapturer;
    impl ScreenCapturer for MockScreenCapturer {
        fn capture_screen_at_region(
            &self,
            _region: &ScreenRegion,
        ) -> anyhow::Result<CaptureBuffer> {
            let raw_data = vec![255u8; 100 * 100 * 4];
            Ok(CaptureBuffer::build_from_raw_data(1.0, 100, 100, raw_data))
        }
    }

    struct MockMouseProvider;
    impl MousePositionProvider for MockMouseProvider {
        fn get_current_mouse_position(&self) -> Result<ScreenRegion, String> {
            Ok(ScreenRegion::at_coordinates(0, 0))
        }
    }

    struct MockOcrService;
    #[async_trait::async_trait]
    impl OcrService for MockOcrService {
        async fn extract_text_from_image(
            &self,
            _image: &image::DynamicImage,
        ) -> anyhow::Result<OcrResult> {
            Ok(OcrResult {
                text_blocks: vec![],
                full_text: "test".to_string(),
            })
        }
    }

    struct MockSearchProvider;
    #[async_trait::async_trait]
    impl ReverseImageSearchProvider for MockSearchProvider {
        async fn perform_search(&self, _buffer: &CaptureBuffer) -> anyhow::Result<String> {
            Ok("https://test.com/search".to_string())
        }
    }

    fn create_test_orchestrator() -> AppOrchestrator {
        AppOrchestrator::build(
            Arc::new(MockScreenCapturer),
            Arc::new(MockMouseProvider),
            Arc::new(MockOcrService),
            Arc::new(MockSearchProvider),
            UserSettings::default(),
        )
    }

    #[test]
    fn test_build_creates_orchestrator_with_correct_initial_state() {
        let orchestrator = create_test_orchestrator();

        assert_eq!(orchestrator.windows.len(), 0);
        assert!(orchestrator.main_window_id.is_none());
        assert!(orchestrator.settings_window_id.is_none());
        assert!(orchestrator.temp_settings.is_none());
        assert!(!orchestrator.status.is_empty());
    }

    #[test]
    fn test_handle_capture_error_updates_status() {
        let mut orchestrator = create_test_orchestrator();
        let error_message = "Test error".to_string();

        let _ = orchestrator.handle_capture_error(error_message.clone());

        assert_eq!(orchestrator.status, error_message);
    }

    #[test]
    fn test_handle_ocr_service_ready_updates_service() {
        let mut orchestrator = create_test_orchestrator();
        let new_service = Arc::new(MockOcrService) as Arc<dyn OcrService>;

        let _ = orchestrator.handle_ocr_service_ready(new_service);

        assert!(orchestrator.status.contains("Ready"));
    }

    #[test]
    fn test_handle_ocr_service_failed_updates_status() {
        let mut orchestrator = create_test_orchestrator();
        let error = "OCR initialization failed".to_string();

        let _ = orchestrator.handle_ocr_service_failed(error.clone());

        assert!(orchestrator.status.contains("OCR initialization failed"));
    }

    #[test]
    fn test_update_settings_modifies_temp_settings() {
        let mut orchestrator = create_test_orchestrator();
        orchestrator.temp_settings = Some(UserSettings::default());

        let new_url = "https://new.search.com?q={}".to_string();
        let _ = orchestrator.update(OrchestratorMessage::UpdateSearchUrl(new_url.clone()));

        assert_eq!(
            orchestrator
                .temp_settings
                .unwrap()
                .image_search_url_template,
            new_url
        );
    }

    #[test]
    fn test_update_hotkey_modifies_temp_settings() {
        let mut orchestrator = create_test_orchestrator();
        orchestrator.temp_settings = Some(UserSettings::default());

        let new_hotkey = "Ctrl+Shift+C".to_string();
        let _ = orchestrator.update(OrchestratorMessage::UpdateHotkey(new_hotkey.clone()));

        assert_eq!(
            orchestrator.temp_settings.unwrap().capture_hotkey,
            new_hotkey
        );
    }

    #[test]
    fn test_update_theme_modifies_temp_settings() {
        let mut orchestrator = create_test_orchestrator();
        orchestrator.temp_settings = Some(UserSettings::default());

        let _ = orchestrator.update(OrchestratorMessage::UpdateTheme(ThemeMode::Light));

        assert!(matches!(
            orchestrator.temp_settings.unwrap().theme_mode,
            ThemeMode::Light
        ));
    }

    #[test]
    fn test_get_window_title_returns_correct_title() {
        let orchestrator = create_test_orchestrator();
        let id = Id::unique();

        let title = orchestrator.get_window_title(id);

        assert_eq!(title, "Circle to Search");
    }
}
