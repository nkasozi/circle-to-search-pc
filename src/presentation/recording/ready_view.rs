use iced::widget::{column, container, image, row, text, Space};
use iced::{Alignment, Element, Length};

mod thumbnail;

use thumbnail::render_thumbnail;

use crate::presentation::design::components::{icon_button, labeled_button, with_tooltip};
use crate::presentation::design::{
    font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const SAVED_TITLE: &str = "Recording saved";
const PLAY_LABEL: &str = "Play";
const SHOW_LABEL: &str = "Show in folder";
const DELETE_TOOLTIP: &str = "Delete recording";
const CLOSE_TOOLTIP: &str = "Close";
const OPEN_SETTINGS_LABEL: &str = "Open settings";
const TRY_AGAIN_LABEL: &str = "Try again";
const FILE_DETAILS_SPACING: f32 = 2.0;

#[derive(Debug, Clone)]
pub enum ReadyCardContent {
    Saved {
        file_name: String,
        details: String,
        thumbnail: Option<image::Handle>,
    },
    Failed {
        title: String,
        reason: String,
        offers_permission_settings: bool,
    },
}

#[derive(Debug, Clone)]
pub enum RecordingReadyMessage {
    Play,
    Reveal,
    Delete,
    Dismiss,
    RecordAgain,
    OpenPermissionSettings,
}

#[derive(Debug, Clone)]
pub struct RecordingReadyView {
    pub content: ReadyCardContent,
}

impl RecordingReadyView {
    pub fn render(&self) -> Element<'_, RecordingReadyMessage> {
        let body = match &self.content {
            ReadyCardContent::Saved {
                file_name,
                details,
                thumbnail,
            } => render_saved(file_name, details, thumbnail.as_ref()),
            ReadyCardContent::Failed {
                title,
                reason,
                offers_permission_settings,
            } => render_failed(title, reason, *offers_permission_settings),
        };
        container(body)
            .padding(spacing::LARGE)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(surfaces::floating_card)
            .into()
    }
}

fn render_header<'a>(
    title: &'a str,
    title_icon: Icon,
    tint: IconTint,
) -> Element<'a, RecordingReadyMessage> {
    row![
        icon(title_icon, icon_size::MEDIUM, tint),
        text(title).size(font_size::HEADING),
        Space::new().width(Length::Fill),
        with_tooltip(
            icon_button(
                Icon::Close,
                RecordingReadyMessage::Dismiss,
                ButtonTone::Ghost,
                IconTint::TextSecondary
            ),
            CLOSE_TOOLTIP,
            iced::widget::tooltip::Position::Left
        ),
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center)
    .into()
}

fn render_saved<'a>(
    file_name: &'a str,
    details: &'a str,
    thumbnail: Option<&'a image::Handle>,
) -> Element<'a, RecordingReadyMessage> {
    column![
        render_header(SAVED_TITLE, Icon::Check, IconTint::Success),
        render_thumbnail(thumbnail),
        column![
            text(file_name).size(font_size::BODY),
            text(details)
                .size(font_size::CAPTION)
                .style(surfaces::muted_text),
        ]
        .spacing(FILE_DETAILS_SPACING),
        render_saved_actions(),
    ]
    .spacing(spacing::MEDIUM)
    .into()
}

fn render_saved_actions<'a>() -> Element<'a, RecordingReadyMessage> {
    row![
        labeled_button(
            Icon::Play,
            PLAY_LABEL,
            RecordingReadyMessage::Play,
            ButtonTone::Primary,
            iced::Color::WHITE
        ),
        labeled_button(
            Icon::Folder,
            SHOW_LABEL,
            RecordingReadyMessage::Reveal,
            ButtonTone::Secondary,
            IconTint::TextPrimary
        ),
        Space::new().width(Length::Fill),
        with_tooltip(
            icon_button(
                Icon::Trash,
                RecordingReadyMessage::Delete,
                ButtonTone::Danger,
                IconTint::Danger
            ),
            DELETE_TOOLTIP,
            iced::widget::tooltip::Position::Top
        ),
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center)
    .into()
}

fn render_failed<'a>(
    title: &'a str,
    reason: &'a str,
    offers_permission_settings: bool,
) -> Element<'a, RecordingReadyMessage> {
    let primary_action = match offers_permission_settings {
        true => labeled_button(
            Icon::Shield,
            OPEN_SETTINGS_LABEL,
            RecordingReadyMessage::OpenPermissionSettings,
            ButtonTone::Primary,
            iced::Color::WHITE,
        ),
        false => labeled_button(
            Icon::Refresh,
            TRY_AGAIN_LABEL,
            RecordingReadyMessage::RecordAgain,
            ButtonTone::Primary,
            iced::Color::WHITE,
        ),
    };
    column![
        render_header(title, Icon::Close, IconTint::Danger),
        text(reason)
            .size(font_size::BODY)
            .style(surfaces::secondary_text),
        Space::new().height(Length::Fill),
        row![primary_action, Space::new().width(Length::Fill)],
    ]
    .spacing(spacing::MEDIUM)
    .into()
}
