use std::path::PathBuf;
use std::time::SystemTime;

use iced::widget::{column, container, Space};
use iced::{Element, Length};

use crate::core::models::RecordingSummary;
use crate::presentation::design::{spacing, surfaces};

mod action_cards;
mod header_footer;
mod recent_recordings;

const HOME_CONTENT_MAXIMUM_WIDTH: f32 = 640.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTone {
    Neutral,
    Busy,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct HomeViewModel<'a> {
    pub capture_hotkey: &'a str,
    pub recording_hotkey: &'a str,
    pub status_text: &'a str,
    pub status_tone: StatusTone,
    pub show_home_on_launch: bool,
    pub is_recording: bool,
    pub recent_recordings: &'a [RecordingSummary],
    pub now: SystemTime,
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    StartCapture,
    StartRecording,
    OpenSettings,
    ToggleShowHomeOnLaunch(bool),
    OpenRecording(PathBuf),
    RevealRecording(PathBuf),
    OpenRecordingsFolder,
}

pub fn render_home<'a>(model: HomeViewModel<'a>) -> Element<'a, HomeMessage> {
    let content = column![
        header_footer::render_header(),
        action_cards::render_action_cards(&model),
        recent_recordings::render_recent_recordings(model.recent_recordings, model.now),
        Space::new().height(Length::Fill),
        header_footer::render_footer(&model),
    ]
    .spacing(spacing::EXTRA_LARGE)
    .padding(spacing::HUGE)
    .max_width(HOME_CONTENT_MAXIMUM_WIDTH);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .style(surfaces::window_background)
        .into()
}
