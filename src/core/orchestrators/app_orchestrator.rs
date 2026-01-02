use std::collections::HashMap;
use std::sync::Arc;

use iced::{Alignment, Background, Color, Element, Length, Point, Rectangle, Size, Task};
use iced::widget::{button, column, container, text};
use iced::window::{self, Id};
use mouse_position::mouse_position::Mouse;
use base64::Engine;

use crate::core::models::{CaptureBuffer, OcrResult, ScreenRegion};
use crate::core::interfaces::adapters::OcrService;
use crate::core::interfaces::ports::{MousePositionProvider, ScreenCapturer};
use crate::presentation::{CaptureView, CaptureViewMessage};
use crate::user_settings;
use crate::app_theme;
use crate::global_constants;
use crate::ports::GlobalKeyboardEvent;

pub enum AppWindow {
    Main,
    CaptureOverlay(CaptureView),
    InteractiveOcr(crate::presentation::InteractiveOcrView),
    Settings,
}

pub struct AppOrchestrator {
    screen_capturer: Arc<dyn ScreenCapturer>,
    #[allow(dead_code)]
    mouse_provider: Arc<dyn MousePositionProvider>,
    ocr_service: Arc<dyn OcrService>,
    windows: HashMap<Id, AppWindow>,
    main_window_id: Option<Id>,
    status: String,
    settings: user_settings::UserSettings,
    settings_window_id: Option<Id>,
    temp_settings: Option<user_settings::UserSettings>,
}

#[derive(Clone)]
pub enum OrchestratorMessage {
    OpenMainWindow,
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
    UpdateTheme(user_settings::ThemeMode),
    SaveSettings,
    RestartApp,
}

impl std::fmt::Debug for OrchestratorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorMessage::OpenMainWindow => write!(f, "OpenMainWindow"),
            OrchestratorMessage::CaptureScreen => write!(f, "CaptureScreen"),
            OrchestratorMessage::PerformCapture => write!(f, "PerformCapture"),
            OrchestratorMessage::OpenCaptureOverlay(x, y, _) => write!(f, "OpenCaptureOverlay({}, {})", x, y),
            OrchestratorMessage::CaptureError(e) => write!(f, "CaptureError({})", e),
            OrchestratorMessage::CaptureOverlayMessage(id, _) => write!(f, "CaptureOverlayMessage({:?})", id),
            OrchestratorMessage::ConfirmSelection(id) => write!(f, "ConfirmSelection({:?})", id),
            OrchestratorMessage::ShowCroppedImage(_, rect) => write!(f, "ShowCroppedImage({:?})", rect),
            OrchestratorMessage::ProcessOcr(id, _) => write!(f, "ProcessOcr({:?})", id),
            OrchestratorMessage::OcrComplete(id, result) => write!(f, "OcrComplete({:?}, {:?})", id, result.is_ok()),
            OrchestratorMessage::OcrServiceReady(_) => write!(f, "OcrServiceReady"),
            OrchestratorMessage::OcrServiceFailed(e) => write!(f, "OcrServiceFailed({})", e),
            OrchestratorMessage::InteractiveOcrMessage(id, _) => write!(f, "InteractiveOcrMessage({:?})", id),
            OrchestratorMessage::PerformImageSearch(id, _) => write!(f, "PerformImageSearch({:?})", id),
            OrchestratorMessage::CloseWindow(id) => write!(f, "CloseWindow({:?})", id),
            OrchestratorMessage::WindowClosed(id) => write!(f, "WindowClosed({:?})", id),
            OrchestratorMessage::Keyboard(event) => write!(f, "Keyboard({:?})", event),
            OrchestratorMessage::OpenSettings => write!(f, "OpenSettings"),
            OrchestratorMessage::UpdateSearchUrl(_) => write!(f, "UpdateSearchUrl"),
            OrchestratorMessage::UpdateHotkey(_) => write!(f, "UpdateHotkey"),
            OrchestratorMessage::UpdateTheme(_) => write!(f, "UpdateTheme"),
            OrchestratorMessage::SaveSettings => write!(f, "SaveSettings"),
            OrchestratorMessage::RestartApp => write!(f, "RestartApp"),
        }
    }
}

impl AppOrchestrator {
    pub fn build(
        screen_capturer: Arc<dyn ScreenCapturer>,
        mouse_provider: Arc<dyn MousePositionProvider>,
        ocr_service: Arc<dyn OcrService>,
        settings: user_settings::UserSettings,
    ) -> Self {
        Self {
            screen_capturer,
            mouse_provider,
            ocr_service,
            windows: HashMap::new(),
            main_window_id: None,
            status: "Initializing OCR service...".to_string(),
            settings,
            settings_window_id: None,
            temp_settings: None,
        }
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
            OrchestratorMessage::SaveSettings => {
                return self.handle_save_settings();
            }
            OrchestratorMessage::RestartApp => {
                return self.handle_restart_app();
            }
        }

        log::debug!("[ORCHESTRATOR] No task to return, ending update");
        Task::none()
    }

    pub fn render_view(&self, window_id: Id) -> Element<'_, OrchestratorMessage> {
        match self.windows.get(&window_id) {
            Some(AppWindow::Main) => self.render_main_window(),
            Some(AppWindow::CaptureOverlay(capture_view)) => {
                capture_view.render_ui().map(move |msg| OrchestratorMessage::CaptureOverlayMessage(window_id, msg))
            }
            Some(AppWindow::InteractiveOcr(ocr_view)) => {
                ocr_view.render_ui().map(move |msg| OrchestratorMessage::InteractiveOcrMessage(window_id, msg))
            }
            Some(AppWindow::Settings) => self.render_settings_window(),
            None => text("Loading...").into(),
        }
    }

    fn handle_open_main_window(&mut self) -> Task<OrchestratorMessage> {
        log::debug!("[ORCHESTRATOR] Opening main window");
        if self.windows.is_empty() {
            let (id, task) = window::open(window::Settings {
                size: Size::new(700.0, 600.0),
                position: window::Position::Centered,
                resizable: false,
                ..Default::default()
            });

            self.main_window_id = Some(id);
            self.windows.insert(id, AppWindow::Main);
            log::info!("[ORCHESTRATOR] Main window created with ID: {:?}", id);
            return task.discard();
        }

        log::warn!("[ORCHESTRATOR] Main window already exists, skipping creation");
        Task::none()
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
            })
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
                    log::info!("[ORCHESTRATOR] Screen captured successfully, buffer size: {}x{}", capture_buffer.width, capture_buffer.height);
                    OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer)
                }
                Err(e) => {
                    log::error!("[ORCHESTRATOR] Screen capture failed: {}. If multiple instances are running, this may be expected.", e);
                    OrchestratorMessage::CaptureError(format!("Capture failed: {}. Try closing other instances.", e))
                }
            }
        })
    }

    fn handle_open_capture_overlay(&mut self, mouse_x: i32, mouse_y: i32, capture_buffer: CaptureBuffer) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Opening capture overlay at ({}, {})", mouse_x, mouse_y);
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
                self.windows.insert(id, AppWindow::CaptureOverlay(capture_view));
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
        if let Some((id, AppWindow::CaptureOverlay(_))) =
            self.windows.iter().find(|(_, w)| matches!(w, AppWindow::CaptureOverlay(_))) {
            log::debug!("[ORCHESTRATOR] Closing overlay window: {:?}", id);
            return window::close(*id);
        }
        log::debug!("[ORCHESTRATOR] No overlay window found to close");
        self.status = "Ready - Press Alt+Shift+S to capture".to_string();
        Task::none()
    }

    fn handle_capture_overlay_message(&mut self, window_id: Id, capture_msg: CaptureViewMessage) -> Task<OrchestratorMessage> {
        log::debug!("[ORCHESTRATOR] Received overlay message for window {:?}: {:?}", window_id, capture_msg);
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
        log::info!("[ORCHESTRATOR] Confirming selection from overlay {:?}", overlay_id);

        if let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get(&overlay_id) {
            if let Some(selection_rect) = capture_view.get_selected_region() {
                log::info!("[ORCHESTRATOR] Selection region: {:?}", selection_rect);
                let capture_buffer = capture_view.get_capture_buffer().clone();

                self.status = "Processing selection...".to_string();
                return Task::batch(vec![
                    window::close(overlay_id),
                    Task::done(OrchestratorMessage::ShowCroppedImage(capture_buffer, selection_rect))
                ]);
            }
            log::warn!("[ORCHESTRATOR] No selection region found");
        } else {
            log::warn!("[ORCHESTRATOR] Overlay window not found");
        }

        window::close(overlay_id)
    }

    fn handle_show_cropped_image(&mut self, capture_buffer: CaptureBuffer, selection_rect: Rectangle) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Showing cropped image from selection: {:?}", selection_rect);

        let cropped_buffer = capture_buffer.crop_region(
            selection_rect.x as u32,
            selection_rect.y as u32,
            selection_rect.width as u32,
            selection_rect.height as u32,
        );

        match cropped_buffer {
            Ok(buffer) => {
                log::info!("[ORCHESTRATOR] Successfully cropped image: {}x{}", buffer.width, buffer.height);

                let (id, task) = window::open(window::Settings {
                    size: Size::new(
                        (buffer.width as f32).min(1200.0),
                        (buffer.height as f32).min(800.0)
                    ),
                    position: window::Position::Centered,
                    resizable: true,
                    ..Default::default()
                });

                let view = crate::presentation::InteractiveOcrView::build(buffer.clone(), self.settings.theme_mode.clone());
                self.windows.insert(id, AppWindow::InteractiveOcr(view));
                self.status = "Processing OCR...".to_string();

                return Task::batch(vec![
                    task.discard(),
                    Task::done(OrchestratorMessage::ProcessOcr(id, buffer))
                ]);
            }
            Err(e) => {
                log::error!("[ORCHESTRATOR] Failed to crop image: {}", e);
                self.status = format!("Error cropping image: {}", e);
            }
        }
        Task::none()
    }

    fn handle_process_ocr(&mut self, window_id: Id, buffer: CaptureBuffer) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Starting OCR processing for window {:?}", window_id);

        let ocr_service = self.ocr_service.clone();
        let width = buffer.width;
        let height = buffer.height;

        Task::future(async move {
            log::debug!("[OCR] Converting capture buffer to dynamic image {}x{}", width, height);

            let dynamic_image = match image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, buffer.raw_data.clone())
                    .expect("Failed to create image from raw data")
            ) {
                img => img,
            };

            log::debug!("[OCR] Running OCR on image");
            match ocr_service.extract_text_from_image(&dynamic_image).await {
                Ok(result) => {
                    log::info!("[OCR] OCR completed successfully. Found {} text blocks", result.text_blocks.len());
                    OrchestratorMessage::OcrComplete(window_id, Ok(result))
                }
                Err(e) => {
                    log::error!("[OCR] OCR failed: {}", e);
                    OrchestratorMessage::OcrComplete(window_id, Err(e.to_string()))
                }
            }
        })
    }

    fn handle_ocr_complete(&mut self, window_id: Id, result: Result<OcrResult, String>) -> Task<OrchestratorMessage> {
        match result {
            Ok(ocr_result) => {
                log::info!("[ORCHESTRATOR] OCR complete for window {:?}: {} text blocks found", window_id, ocr_result.text_blocks.len());

                if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
                    view.set_ocr_result(ocr_result);
                    self.status = "OCR complete".to_string();
                }
            }
            Err(e) => {
                log::error!("[ORCHESTRATOR] OCR failed for window {:?}: {}", window_id, e);
                self.status = format!("OCR failed: {}", e);
            }
        }
        Task::none()
    }

    fn handle_ocr_service_ready(&mut self, service: Arc<dyn OcrService>) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] OCR service is ready");
        self.ocr_service = service;
        self.status = "Ready - Press Alt+Shift+S to capture".to_string();
        Task::none()
    }

    fn handle_ocr_service_failed(&mut self, error: String) -> Task<OrchestratorMessage> {
        log::error!("[ORCHESTRATOR] OCR service initialization failed: {}", error);
        self.status = format!("OCR initialization failed: {}", error);
        Task::none()
    }

    fn handle_interactive_ocr_message(&mut self, window_id: Id, ocr_msg: crate::presentation::InteractiveOcrMessage) -> Task<OrchestratorMessage> {
        log::debug!("[ORCHESTRATOR] Received OCR message for window {:?}: {:?}", window_id, ocr_msg);

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
                    OrchestratorMessage::InteractiveOcrMessage(window_id, crate::presentation::InteractiveOcrMessage::HideToast)
                });
            }
            _ => {}
        }
        Task::none()
    }

    fn handle_perform_image_search(&mut self, window_id: Id, buffer: CaptureBuffer) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Starting image search for window {:?}", window_id);

        let search_url_template = self.settings.image_search_url_template.clone();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                crate::presentation::InteractiveOcrMessage::SearchUploading
            )),
            Task::future(async move {
                let search_future = async {
                    let temp_dir = std::env::temp_dir();
                    let image_path = temp_dir.join("circle_to_search_image.png");

                    log::debug!("[SEARCH] Saving image to temp: {:?}", image_path);

                    let img = ::image::DynamicImage::ImageRgba8(
                        ::image::RgbaImage::from_raw(
                            buffer.width,
                            buffer.height,
                            buffer.raw_data.clone(),
                        )
                        .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?
                    );

                    img.save(&image_path)?;

                    log::info!("[SEARCH] Uploading image to imgbb");

                    let image_data = tokio::fs::read(&image_path).await?;
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

                    let client = reqwest::Client::new();
                    let form = reqwest::multipart::Form::new()
                        .text("image", base64_image)
                        .text("expiration", global_constants::IMGBB_EXPIRATION_SECONDS);

                    let upload_url = format!("{}?key={}", global_constants::IMGBB_API_URL, global_constants::IMGBB_API_KEY);
                    let response = client
                        .post(&upload_url)
                        .multipart(form)
                        .send()
                        .await?;

                    let response_text = response.text().await?;
                    log::debug!("[SEARCH] imgbb response: {}", response_text);

                    let json: serde_json::Value = serde_json::from_str(&response_text)?;

                    let image_url = json["data"]["url"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Failed to extract image URL from imgbb response"))?;

                    let encoded_url = urlencoding::encode(image_url);
                    let search_url = search_url_template.replace("{}", &encoded_url);

                    log::info!("[SEARCH] Opening Google reverse image search");
                    log::debug!("[SEARCH] Image URL: {}", image_url);
                    log::debug!("[SEARCH] Search URL: {}", search_url);

                    open::that(&search_url)?;

                    Ok::<(), anyhow::Error>(())
                };

                let timeout_duration = std::time::Duration::from_secs(30);
                match tokio::time::timeout(timeout_duration, search_future).await {
                    Ok(Ok(())) => {
                        log::info!("[ORCHESTRATOR] Image search completed successfully");
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchCompleted
                        )
                    }
                    Ok(Err(e)) => {
                        log::error!("[ORCHESTRATOR] Image search failed: {}", e);
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchFailed(e.to_string())
                        )
                    }
                    Err(_) => {
                        log::error!("[ORCHESTRATOR] Image search timed out after 30 seconds");
                        OrchestratorMessage::InteractiveOcrMessage(
                            window_id,
                            crate::presentation::InteractiveOcrMessage::SearchFailed(
                                "Search timed out after 30 seconds".to_string()
                            )
                        )
                    }
                }
            })
        ])
    }

    fn handle_window_closed(&mut self, id: Id) -> Task<OrchestratorMessage> {
        log::info!("[ORCHESTRATOR] Window closed: {:?}", id);
        let was_ocr_window = matches!(self.windows.get(&id), Some(AppWindow::InteractiveOcr(_)));
        self.windows.remove(&id);
        if Some(id) == self.settings_window_id {
            self.settings_window_id = None;
            self.temp_settings = None;
        }
        log::debug!("[ORCHESTRATOR] Removed window from tracking. Remaining: {}", self.windows.len());
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

    fn render_main_window(&self) -> Element<'_, OrchestratorMessage> {
        let theme = app_theme::get_theme(&self.settings.theme_mode);

        let title = text("Circle to Search - Desktop Edition")
            .size(40);

        let btn = button(text("📸 Capture Screen"))
            .padding([18, 40])
            .style(|theme, status| {
                app_theme::primary_button_style(theme, status)
            })
            .on_press(OrchestratorMessage::CaptureScreen);

        let settings_btn = button(text("⚙️ Settings").size(20))
            .padding([18, 40])
            .style(|theme, status| {
                app_theme::purple_button_style(theme, status)
            })
            .on_press(OrchestratorMessage::OpenSettings);

        let content = column![
            title,
            text("").size(20),
            text("How to Use:").size(18),
            text("• Click the button below to capture your screen"),
            text("• Or press Alt+Shift+S anywhere on your computer"),
            text("• Press Escape to close overlay"),
            text("").size(20),
            btn,
            text("").size(10),
            text(format!("Status: {}", &self.status)),
            text("").size(20),
            settings_btn,
        ]
        .spacing(10)
        .padding(50)
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

    fn render_settings_window(&self) -> Element<'_, OrchestratorMessage> {
        use iced::widget::{text_input, pick_list};

        let theme = app_theme::get_theme(&self.settings.theme_mode);
        let temp = self.temp_settings.as_ref().unwrap_or(&self.settings);

        let title = text("Settings")
            .size(32)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        let search_url_label = text("Image Search URL Template:").size(16);
        let search_url_input = text_input("https://lens.google.com/uploadbyurl?url={}", &temp.image_search_url_template)
            .on_input(OrchestratorMessage::UpdateSearchUrl)
            .padding(10);

        let hotkey_label = text("Capture Hotkey:").size(16);
        let hotkey_input = text_input("Alt+Shift+S", &temp.capture_hotkey)
            .on_input(OrchestratorMessage::UpdateHotkey)
            .padding(10);
        let hotkey_warning = text("⚠️ Changing hotkey requires app restart")
            .size(12)
            .style(|_theme: &iced::Theme| {
                iced::widget::text::Style {
                    color: Some(Color::from_rgb(1.0, 0.7, 0.0)),
                }
            });

        let theme_label = text("Theme:").size(16);
        let theme_picker = pick_list(
            vec![user_settings::ThemeMode::Dark, user_settings::ThemeMode::Light],
            Some(temp.theme_mode.clone()),
            OrchestratorMessage::UpdateTheme
        )
        .padding(10);

        let save_btn = button(text("💾 Save Settings"))
            .padding([15, 40])
            .style(|theme, status| {
                app_theme::primary_button_style(theme, status)
            })
            .on_press(OrchestratorMessage::SaveSettings);

        let content = column![
            title,
            text("").size(20),
            search_url_label,
            search_url_input,
            text("").size(10),
            hotkey_label,
            hotkey_input,
            hotkey_warning,
            text("").size(10),
            theme_label,
            theme_picker,
            text("").size(30),
            save_btn,
        ]
        .spacing(8)
        .padding(30)
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
}
