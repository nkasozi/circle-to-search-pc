#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod global_constants;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use iced::{Element, Task, Theme};
use iced::widget::{button, column, container, text};
use iced::{Length, Alignment};

use core::models::ScreenRegion;
use core::ports::{MousePositionProvider, ScreenCapturer};
use global_constants::{
    APPLICATION_TITLE,
    LOG_TAG_APP,
    MESSAGE_CAPTURE_CREATED,
    MESSAGE_CAPTURE_FAILED,
    MESSAGE_INITIALIZING_APP,
    MESSAGE_MONITOR_FOUND,
    MESSAGE_OPENING_CAPTURE,
    MESSAGE_STARTING_APP,
    USER_MESSAGE_INFO_OPENING,
    USER_MESSAGE_SUCCESS_OPENED,
};
use infrastructure::{
    GlobalKeyboardEvent,
    GlobalKeyboardListener,
    SystemMousePositionProvider,
    XcapScreenCapturer,
};
use presentation::CaptureView;

fn main() -> iced::Result {
    env_logger::init();
    log::info!("{} {}", LOG_TAG_APP, MESSAGE_STARTING_APP);

    iced::run("Circle to Search", update, view)
}

fn update(app: &mut CircleToSearchApp, message: AppMessage) -> Task<AppMessage> {
    app.update(message)
}

fn view(app: &CircleToSearchApp) -> Element<AppMessage> {
    app.view()
}

fn subscription(_app: &CircleToSearchApp) -> iced::Subscription<AppMessage> {
    iced::Subscription::run(GlobalKeyboardListener::create_event_stream)
        .map(AppMessage::KeyboardEvent)
}

pub struct CircleToSearchApp {
    screen_capturer: Arc<dyn ScreenCapturer>,
    mouse_position_provider: Arc<dyn MousePositionProvider>,
    capture_window: Option<CaptureView>,
    status_message: String,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    StartCapture,
    KeyboardEvent(GlobalKeyboardEvent),
    CloseCapture,
}

impl Default for CircleToSearchApp {
    fn default() -> Self {
        log::info!("{} {}", LOG_TAG_APP, MESSAGE_INITIALIZING_APP);

        Self {
            screen_capturer: Arc::new(XcapScreenCapturer::initialize()),
            mouse_position_provider: Arc::new(SystemMousePositionProvider::initialize()),
            capture_window: None,
            status_message: "Ready. Press 'Start Capture' or Alt+Shift+S".to_string(),
        }
    }
}

impl CircleToSearchApp {
    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::StartCapture => self.start_capture_process(),
            AppMessage::KeyboardEvent(keyboard_event) => self.handle_keyboard_event(keyboard_event),
            AppMessage::CloseCapture => self.close_capture_window(),
        }
    }

    fn start_capture_process(&mut self) -> Task<AppMessage> {
        log::debug!("{} {}", LOG_TAG_APP, MESSAGE_OPENING_CAPTURE);
        self.status_message = USER_MESSAGE_INFO_OPENING.to_string();

        if self.capture_window.is_some() {
            log::debug!("{} capture already active, ignoring request", LOG_TAG_APP);
            return Task::none();
        }

        let screen_region = self.determine_capture_region();
        let capture_result = self.screen_capturer.capture_screen_at_region(&screen_region);

        match capture_result {
            Ok(capture_buffer) => {
                log::debug!(
                    "{} {} ({}, {})",
                    LOG_TAG_APP,
                    MESSAGE_MONITOR_FOUND,
                    screen_region.x_position,
                    screen_region.y_position
                );

                let capture_view = CaptureView::build_with_capture_buffer(capture_buffer);

                log::info!("{} {}", LOG_TAG_APP, MESSAGE_CAPTURE_CREATED);
                self.status_message = USER_MESSAGE_SUCCESS_OPENED.to_string();
                self.capture_window = Some(capture_view);

                Task::none()
            }
            Err(error) => {
                log::error!("{} {}: {}", LOG_TAG_APP, MESSAGE_CAPTURE_FAILED, error);
                self.status_message = format!("Capture failed: {}", error);
                Task::none()
            }
        }
    }

    fn close_capture_window(&mut self) -> Task<AppMessage> {
        log::debug!("{} closing capture window", LOG_TAG_APP);
        self.capture_window = None;
        self.status_message = "Capture closed. Ready for next capture.".to_string();
        Task::none()
    }

    fn handle_keyboard_event(&mut self, event: GlobalKeyboardEvent) -> Task<AppMessage> {
        match event {
            GlobalKeyboardEvent::CaptureHotkeyPressed => self.start_capture_process(),
            GlobalKeyboardEvent::EscapePressed => self.close_capture_window(),
        }
    }

    fn determine_capture_region(&self) -> ScreenRegion {
        self.mouse_position_provider
            .get_current_mouse_position()
            .unwrap_or_else(|_| ScreenRegion::default_origin())
    }

    pub fn view(&self) -> Element<AppMessage> {
        if let Some(capture_view) = &self.capture_window {
            return self.build_capture_overlay_view(capture_view);
        }

        self.build_main_window_view()
    }

    fn build_main_window_view(&self) -> Element<AppMessage> {
        let title = text("Circle to Search - Desktop")
            .size(32)
            .width(Length::Fill);

        let instructions = column![
            text("How to use:").size(20),
            text(""),
            text("1. Click 'Start Capture' button below"),
            text("2. Or press Alt+Shift+S anywhere"),
            text("3. The screen will be captured"),
            text("4. Press Escape to close capture"),
            text(""),
        ]
        .spacing(5);

        let capture_button = button(
            text("Start Capture")
                .size(20)
                .width(Length::Fill)
        )
        .padding(15)
        .width(Length::Fixed(200.0))
        .on_press(AppMessage::StartCapture);

        let status_text = text(&self.status_message)
            .size(14)
            .style(|_theme| text::Style {
                color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            });

        let content = column![
            title,
            text("").size(10),
            instructions,
            capture_button,
            text("").size(20),
            status_text,
        ]
        .spacing(10)
        .padding(40)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    fn build_capture_overlay_view<'a>(&'a self, capture_view: &'a CaptureView) -> Element<'a, AppMessage> {
        let close_button = button(text("Close (Esc)"))
            .padding(10)
            .on_press(AppMessage::CloseCapture);

        let overlay_content = column![
            capture_view.render_ui().map(|_| AppMessage::CloseCapture),
            container(close_button)
                .padding(20)
                .center_x(Length::Fill)
        ]
        .spacing(0);

        container(overlay_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme_fn(_app: &Self) -> Theme {
        Theme::Dark
    }

    pub fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::run(GlobalKeyboardListener::create_event_stream)
            .map(AppMessage::KeyboardEvent)
    }
}