#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod app_theme;
mod core;
mod global_constants;
mod ports;
mod presentation;
mod user_settings;

use std::collections::HashMap;
use std::sync::Arc;

use iced::{Alignment, Background, Element, Length, Point, Rectangle, Size, Task};
use iced::daemon;
use iced::widget::{button, column, container, row, text};
use iced::window::{self, Id};
use mouse_position::mouse_position::Mouse;
use base64::Engine;

use core::models::{CaptureBuffer, OcrResult, ScreenRegion};
use core::interfaces::adapters::OcrService;
use core::interfaces::ports::{MousePositionProvider, ScreenCapturer};
use adapters::TesseractOcrService;
use ports::{
    GlobalKeyboardEvent, GlobalKeyboardListener, SystemMousePositionProvider, XcapScreenCapturer,
};
use presentation::{CaptureView, CaptureViewMessage};

struct DummyOcrService;

#[async_trait::async_trait]
impl OcrService for DummyOcrService {
    async fn extract_text_from_image(&self, _image: &image::DynamicImage) -> anyhow::Result<OcrResult> {
        anyhow::bail!("OCR service not initialized yet")
    }
}

fn main() -> iced::Result {
    env_logger::init();

    daemon(CircleApp::new, CircleApp::update, CircleApp::view)
        .subscription(CircleApp::subscription)
        .run()
}

enum AppWindow {
    Main,
    CaptureOverlay(CaptureView),
    InteractiveOcr(presentation::InteractiveOcrView),
}

struct CircleApp {
    screen_capturer: Arc<dyn ScreenCapturer>,
    #[allow(dead_code)]
    mouse_provider: Arc<dyn MousePositionProvider>,
    ocr_service: Arc<dyn OcrService>,
    windows: HashMap<Id, AppWindow>,
    main_window_id: Option<Id>,
    status: String,
    settings: user_settings::UserSettings,
}

#[derive(Clone)]
enum Message {
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
    InteractiveOcrMessage(Id, presentation::InteractiveOcrMessage),
    PerformImageSearch(Id, CaptureBuffer),
    #[allow(dead_code)]
    CloseWindow(Id),
    WindowClosed(Id),
    Keyboard(GlobalKeyboardEvent),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::OpenMainWindow => write!(f, "OpenMainWindow"),
            Message::CaptureScreen => write!(f, "CaptureScreen"),
            Message::PerformCapture => write!(f, "PerformCapture"),
            Message::OpenCaptureOverlay(x, y, _) => write!(f, "OpenCaptureOverlay({}, {})", x, y),
            Message::CaptureError(e) => write!(f, "CaptureError({})", e),
            Message::CaptureOverlayMessage(id, _) => write!(f, "CaptureOverlayMessage({:?})", id),
            Message::ConfirmSelection(id) => write!(f, "ConfirmSelection({:?})", id),
            Message::ShowCroppedImage(_, rect) => write!(f, "ShowCroppedImage({:?})", rect),
            Message::ProcessOcr(id, _) => write!(f, "ProcessOcr({:?})", id),
            Message::OcrComplete(id, result) => write!(f, "OcrComplete({:?}, {:?})", id, result.is_ok()),
            Message::OcrServiceReady(_) => write!(f, "OcrServiceReady"),
            Message::OcrServiceFailed(e) => write!(f, "OcrServiceFailed({})", e),
            Message::InteractiveOcrMessage(id, _) => write!(f, "InteractiveOcrMessage({:?})", id),
            Message::PerformImageSearch(id, _) => write!(f, "PerformImageSearch({:?})", id),
            Message::CloseWindow(id) => write!(f, "CloseWindow({:?})", id),
            Message::WindowClosed(id) => write!(f, "WindowClosed({:?})", id),
            Message::Keyboard(event) => write!(f, "Keyboard({:?})", event),
        }
    }
}

impl CircleApp {
    fn new() -> (Self, Task<Message>) {
        log::info!("[APP] Initializing application");

        let settings = user_settings::UserSettings::load()
            .unwrap_or_else(|e| {
                log::warn!("[APP] Failed to load settings: {}, using defaults", e);
                user_settings::UserSettings::default()
            });

        (
            Self {
                screen_capturer: Arc::new(XcapScreenCapturer::initialize()),
                mouse_provider: Arc::new(SystemMousePositionProvider::initialize()),
                ocr_service: Arc::new(DummyOcrService),
                windows: HashMap::new(),
                main_window_id: None,
                status: "Initializing OCR service...".to_string(),
                settings,
            },
            Task::batch(vec![
                Task::done(Message::OpenMainWindow),
                Task::future(async {
                    match TesseractOcrService::build() {
                        Ok(service) => {
                            log::info!("[APP] Tesseract OCR service initialized successfully");
                            Message::OcrServiceReady(Arc::new(service) as Arc<dyn OcrService>)
                        }
                        Err(e) => {
                            log::error!("[APP] Failed to initialize Tesseract OCR service: {}", e);
                            Message::OcrServiceFailed(e.to_string())
                        }
                    }
                })
            ])
        )
    }

    #[allow(dead_code)]
    fn title(&self, _window: Id) -> String {
        "Circle to Search".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        log::info!("[APP] Received message: {:?}", message);

        match message {
            Message::OpenMainWindow => {
                log::debug!("[APP] Opening main window");
                if self.windows.is_empty() {
                    let (id, task) = window::open(window::Settings {
                        size: Size::new(700.0, 500.0),
                        position: window::Position::Centered,
                        resizable: false,
                        ..Default::default()
                    });

                    self.main_window_id = Some(id);
                    self.windows.insert(id, AppWindow::Main);
                    log::info!("[APP] Main window created with ID: {:?}", id);
                    return task.discard();
                } else {
                    log::warn!("[APP] Main window already exists, skipping creation");
                }
            }
            Message::CaptureScreen => {
                log::info!("[APP] Starting capture screen process");
                self.status = "Preparing to capture...".to_string();

                log::debug!("[APP] Waiting 200ms before capture to allow window to update");
                return Task::future(async {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    log::debug!("[APP] Delay complete, triggering PerformCapture");
                    Message::PerformCapture
                });
            }
            Message::PerformCapture => {
                log::info!("[APP] Performing screen capture");
                self.status = "Capturing screen...".to_string();

                let screen_capturer = Arc::clone(&self.screen_capturer);

                return Task::future(async move {
                    log::debug!("[APP] Getting mouse position");
                    let (mouse_x, mouse_y) = match Mouse::get_mouse_position() {
                        Mouse::Position { x, y } => {
                            log::debug!("[APP] Mouse position: ({}, {})", x, y);
                            (x, y)
                        }
                        Mouse::Error => {
                            log::warn!("[APP] Failed to get mouse position, using (0,0)");
                            (0, 0)
                        }
                    };

                    let region = ScreenRegion::at_coordinates(mouse_x, mouse_y);
                    log::debug!("[APP] Capturing screen at region");

                    match screen_capturer.capture_screen_at_region(&region) {
                        Ok(capture_buffer) => {
                            log::info!("[APP] Screen captured successfully");
                            Message::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer)
                        }
                        Err(e) => {
                            log::error!("[APP] Screen capture failed: {}", e);
                            Message::CaptureError(format!("Error: {}", e))
                        }
                    }
                });
            }
            Message::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer) => {
                log::info!("[APP] Opening capture overlay at ({}, {})", mouse_x, mouse_y);
                match xcap::Monitor::from_point(mouse_x, mouse_y) {
                    Ok(monitor) => {
                        log::debug!("[APP] Monitor found, creating overlay window");
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
                        log::info!("[APP] Overlay window created with ID: {:?}", id);

                        return task.discard().chain(window::gain_focus(id));
                    }
                    Err(e) => {
                        log::error!("[APP] Failed to get monitor: {}", e);
                        self.status = format!("Monitor error: {}", e);
                    }
                }
            }
            Message::CaptureError(error_msg) => {
                log::error!("[APP] Capture error: {}", error_msg);
                self.status = error_msg;
            }
            Message::Keyboard(GlobalKeyboardEvent::CaptureHotkeyPressed) => {
                log::info!("[APP] Capture hotkey pressed (Alt+Shift+S)");
                return self.update(Message::CaptureScreen);
            }
            Message::Keyboard(GlobalKeyboardEvent::EscapePressed) => {
                log::info!("[APP] Escape key pressed");
                if let Some((id, AppWindow::CaptureOverlay(_))) =
                    self.windows.iter().find(|(_, w)| matches!(w, AppWindow::CaptureOverlay(_))) {
                    log::debug!("[APP] Closing overlay window: {:?}", id);
                    return window::close(*id);
                }
                log::debug!("[APP] No overlay window found to close");
                self.status = "Ready - Press Alt+Shift+S to capture".to_string();
            }
            Message::CaptureOverlayMessage(window_id, capture_msg) => {
                log::debug!("[APP] Received overlay message for window {:?}: {:?}", window_id, capture_msg);
                if let CaptureViewMessage::ConfirmSelection = capture_msg {
                    log::info!("[APP] Selection confirmed by overlay");
                    return self.update(Message::ConfirmSelection(window_id));
                }
                if let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get_mut(&window_id) {
                    log::debug!("[APP] Updating overlay view state");
                    capture_view.update(capture_msg);
                } else {
                    log::warn!("[APP] Overlay window {:?} not found", window_id);
                }
            }
            Message::ConfirmSelection(overlay_id) => {
                log::info!("[APP] Confirming selection from overlay {:?}", overlay_id);

                if let Some(AppWindow::CaptureOverlay(capture_view)) = self.windows.get(&overlay_id) {
                    if let Some(selection_rect) = capture_view.get_selected_region() {
                        log::info!("[APP] Selection region: {:?}", selection_rect);
                        let capture_buffer = capture_view.get_capture_buffer().clone();

                        self.status = "Processing selection...".to_string();
                        return Task::batch(vec![
                            window::close(overlay_id),
                            Task::done(Message::ShowCroppedImage(capture_buffer, selection_rect))
                        ]);
                    } else {
                        log::warn!("[APP] No selection region found");
                    }
                } else {
                    log::warn!("[APP] Overlay window not found");
                }

                return window::close(overlay_id);
            }
            Message::ShowCroppedImage(capture_buffer, selection_rect) => {
                log::info!("[APP] Showing cropped image from selection: {:?}", selection_rect);

                let cropped_buffer = capture_buffer.crop_region(
                    selection_rect.x as u32,
                    selection_rect.y as u32,
                    selection_rect.width as u32,
                    selection_rect.height as u32,
                );

                match cropped_buffer {
                    Ok(buffer) => {
                        log::info!("[APP] Successfully cropped image: {}x{}", buffer.width, buffer.height);

                        let (id, task) = window::open(window::Settings {
                            size: Size::new(
                                (buffer.width as f32).min(1200.0),
                                (buffer.height as f32).min(800.0)
                            ),
                            position: window::Position::Centered,
                            resizable: true,
                            ..Default::default()
                        });

                        let view = presentation::InteractiveOcrView::build(buffer.clone());
                        self.windows.insert(id, AppWindow::InteractiveOcr(view));
                        self.status = "Processing OCR...".to_string();

                        return Task::batch(vec![
                            task.discard(),
                            Task::done(Message::ProcessOcr(id, buffer))
                        ]);
                    }
                    Err(e) => {
                        log::error!("[APP] Failed to crop image: {}", e);
                        self.status = format!("Error cropping image: {}", e);
                    }
                }
            }
            Message::ProcessOcr(window_id, buffer) => {
                log::info!("[APP] Starting OCR processing for window {:?}", window_id);

                let ocr_service = self.ocr_service.clone();
                let width = buffer.width;
                let height = buffer.height;

                return Task::future(async move {
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
                            Message::OcrComplete(window_id, Ok(result))
                        }
                        Err(e) => {
                            log::error!("[OCR] OCR failed: {}", e);
                            Message::OcrComplete(window_id, Err(e.to_string()))
                        }
                    }
                });
            }
            Message::OcrComplete(window_id, result) => {
                match result {
                    Ok(ocr_result) => {
                        log::info!("[APP] OCR complete for window {:?}: {} text blocks found", window_id, ocr_result.text_blocks.len());

                        if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
                            view.set_ocr_result(ocr_result);
                            self.status = "OCR complete".to_string();
                        }
                    }
                    Err(e) => {
                        log::error!("[APP] OCR failed for window {:?}: {}", window_id, e);
                        self.status = format!("OCR failed: {}", e);
                    }
                }
            }
            Message::OcrServiceReady(service) => {
                log::info!("[APP] OCR service is ready");
                self.ocr_service = service;
                self.status = "Ready - Press Alt+Shift+S to capture".to_string();
            }
            Message::OcrServiceFailed(error) => {
                log::error!("[APP] OCR service initialization failed: {}", error);
                self.status = format!("OCR initialization failed: {}", error);
            }
            Message::InteractiveOcrMessage(window_id, ocr_msg) => {
                log::debug!("[APP] Received OCR message for window {:?}: {:?}", window_id, ocr_msg);

                if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get_mut(&window_id) {
                    view.update(ocr_msg.clone());
                }

                match ocr_msg {
                    presentation::InteractiveOcrMessage::Close => {
                        return window::close(window_id);
                    }
                    presentation::InteractiveOcrMessage::SearchSelected => {
                        if let Some(AppWindow::InteractiveOcr(view)) = self.windows.get(&window_id) {
                            let buffer = view.get_capture_buffer().clone();
                            return Task::done(Message::PerformImageSearch(window_id, buffer));
                        }
                    }
                    _ => {}
                }
            }
            Message::PerformImageSearch(window_id, buffer) => {
                log::info!("[APP] Starting image search for window {:?}", window_id);

                let search_url_template = self.settings.image_search_url_template.clone();

                return Task::batch(vec![
                    Task::done(Message::InteractiveOcrMessage(
                        window_id,
                        presentation::InteractiveOcrMessage::SearchUploading
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
                                log::info!("[APP] Image search completed successfully");
                                Message::InteractiveOcrMessage(
                                    window_id,
                                    presentation::InteractiveOcrMessage::SearchCompleted
                                )
                            }
                            Ok(Err(e)) => {
                                log::error!("[APP] Image search failed: {}", e);
                                Message::InteractiveOcrMessage(
                                    window_id,
                                    presentation::InteractiveOcrMessage::SearchFailed(e.to_string())
                                )
                            }
                            Err(_) => {
                                log::error!("[APP] Image search timed out after 30 seconds");
                                Message::InteractiveOcrMessage(
                                    window_id,
                                    presentation::InteractiveOcrMessage::SearchFailed(
                                        "Search timed out after 30 seconds".to_string()
                                    )
                                )
                            }
                        }
                    })
                ]);
            }
            Message::CloseWindow(id) => {
                log::info!("[APP] Closing window: {:?}", id);
                return window::close(id);
            }
            Message::WindowClosed(id) => {
                log::info!("[APP] Window closed: {:?}", id);
                self.windows.remove(&id);
                log::debug!("[APP] Removed window from tracking. Remaining: {}", self.windows.len());
                self.status = "Ready - Press Alt+Shift+S to capture".to_string();
            }
        }
        log::debug!("[APP] No task to return, ending update");
        Task::none()
    }

    fn view(&self, window_id: Id) -> Element<'_, Message> {
        let _theme = app_theme::get_theme(&self.settings.theme_mode);

        match self.windows.get(&window_id) {
            Some(AppWindow::Main) => self.view_main_window(),
            Some(AppWindow::CaptureOverlay(capture_view)) => {
                capture_view.render_ui().map(move |msg| Message::CaptureOverlayMessage(window_id, msg))
            }
            Some(AppWindow::InteractiveOcr(ocr_view)) => {
                ocr_view.render_ui().map(move |msg| Message::InteractiveOcrMessage(window_id, msg))
            }
            None => text("Loading...").into(),
        }
    }

    fn view_main_window(&self) -> Element<'_, Message> {
        let theme = app_theme::get_theme(&self.settings.theme_mode);

        let title = text("Circle to Search - Desktop Edition")
            .size(40)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        let instructions = column![
            text("").size(30),
            text("How to Use:").size(28).align_x(Alignment::Center),
            text("").size(10),
            text("• Click the button below to capture your screen").size(16).align_x(Alignment::Center),
            text("• Or press Alt+Shift+S anywhere on your computer").size(16).align_x(Alignment::Center),
            text("• Press Escape to close overlay").size(16).align_x(Alignment::Center),
            text("").size(30),
        ]
        .spacing(8)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let btn = button(text("📸 Capture Screen"))
        .height(Length::Fixed(100.0))
        //.width(Length::Fixed(100.0))
        .padding([5, 60])
        .style(|theme, status| {
            app_theme::primary_button_style(theme, status)
        })
        .on_press_with(|| {
            log::info!("[BUTTON] Capture Screen button clicked");
            Message::CaptureScreen
        });

        let status_display = row![
            text("Status: ").size(18),
            text(&self.status).size(18)
        ]
        .spacing(10)
        .width(Length::Fill)
        .align_y(Alignment::Center);

        let status_container = container(status_display)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        let content = column![
            title,
            instructions,
            container(btn).width(Length::Fill).align_x(Alignment::Center),
            text("").size(20),
            status_container
        ]
            .spacing(20)
            .padding(50)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
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

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::Subscription::run(GlobalKeyboardListener::create_event_stream)
                .map(Message::Keyboard),
            iced::event::listen_with(|event, _status, id| {
                if let iced::Event::Window(window::Event::Closed) = event {
                    return Some(Message::WindowClosed(id));
                }
                None
            }),
        ])
    }
}
