#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod core;
mod global_constants;
mod ports;
mod presentation;

use std::collections::HashMap;
use std::sync::Arc;

use iced::{Alignment, Element, Length, Point, Rectangle, Size, Task};
use iced::daemon;
use iced::widget::{button, column, container, row, text};
use iced::window::{self, Id};
use mouse_position::mouse_position::Mouse;

use core::models::{CaptureBuffer, ScreenRegion};
use core::interfaces::ports::{MousePositionProvider, ScreenCapturer};
use ports::{
    GlobalKeyboardEvent, GlobalKeyboardListener, SystemMousePositionProvider, XcapScreenCapturer,
};
use presentation::{CaptureView, CaptureViewMessage};

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
    windows: HashMap<Id, AppWindow>,
    main_window_id: Option<Id>,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    OpenMainWindow,
    CaptureScreen,
    PerformCapture,
    OpenCaptureOverlay(i32, i32, CaptureBuffer),
    CaptureError(String),
    CaptureOverlayMessage(Id, CaptureViewMessage),
    ConfirmSelection(Id),
    ShowCroppedImage(CaptureBuffer, Rectangle),
    InteractiveOcrMessage(Id, presentation::InteractiveOcrMessage),
    #[allow(dead_code)]
    CloseWindow(Id),
    WindowClosed(Id),
    Keyboard(GlobalKeyboardEvent),
}

impl CircleApp {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen_capturer: Arc::new(XcapScreenCapturer::initialize()),
                mouse_provider: Arc::new(SystemMousePositionProvider::initialize()),
                windows: HashMap::new(),
                main_window_id: None,
                status: "Ready - Press Alt+Shift+S to capture".to_string(),
            },
            Task::done(Message::OpenMainWindow),
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

                        let view = presentation::InteractiveOcrView::build(buffer);
                        self.windows.insert(id, AppWindow::InteractiveOcr(view));
                        self.status = "Cropped image ready".to_string();

                        return task.discard();
                    }
                    Err(e) => {
                        log::error!("[APP] Failed to crop image: {}", e);
                        self.status = format!("Error cropping image: {}", e);
                    }
                }
            }
            Message::InteractiveOcrMessage(window_id, ocr_msg) => {
                log::debug!("[APP] Received OCR message for window {:?}: {:?}", window_id, ocr_msg);
                match ocr_msg {
                    presentation::InteractiveOcrMessage::Close => {
                        return window::close(window_id);
                    }
                }
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
        let title = text("Circle to Search - Desktop Edition")
            .size(36)
            .width(Length::Fill);

        let instructions = column![
            text("").size(20),
            text("How to Use:").size(24),
            text(""),
            text("• Click the button below to capture your screen"),
            text("• Or press Alt+Shift+S anywhere on your computer"),
            text("• Press Escape to close overlay"),
            text(""),
        ]
        .spacing(8)
        .width(Length::Fill);

        let btn = button(
            text("📸 Capture Screen")
                .size(24)
                .width(Length::Fill)
        )
        .padding([20, 40])
        .on_press_with(|| {
            log::info!("[BUTTON] Capture Screen button clicked");
            Message::CaptureScreen
        });

        let status_display = row![
            text("Status: ").size(18),
            text(&self.status).size(18)
        ]
        .spacing(10);

        let content = column![title, instructions, btn, text("").size(20), status_display]
            .spacing(20)
            .padding(50)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
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
