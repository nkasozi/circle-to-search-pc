use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Theme};

use super::{HomeMessage, HomeViewModel};
use crate::presentation::design::components::{shortcut_keycaps, KeycapSurface};
use crate::presentation::design::tokens::{radius, tokens_for_theme, DesignTokens};
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const ACTION_CARD_HEIGHT: f32 = 176.0;
const ICON_BADGE_SIZE: f32 = 44.0;
const SEARCH_CARD_TITLE: &str = "Search screen";
const SEARCH_CARD_DESCRIPTION: &str = "Circle anything to search it, copy its text or mark it up";
const RECORD_CARD_TITLE: &str = "Record screen";
const RECORD_CARD_DESCRIPTION: &str = "Tutorials with your camera, voice and click highlights";
const STOP_RECORDING_CARD_TITLE: &str = "Stop recording";
const STOP_RECORDING_CARD_DESCRIPTION: &str = "A recording is in progress. Click to finish it";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BadgeKind {
    Accent,
    Record,
}

struct ActionCardContent<'a> {
    card_icon: Icon,
    badge_kind: BadgeKind,
    title: &'a str,
    description: &'a str,
    accelerator: &'a str,
    on_press: HomeMessage,
}

pub(super) fn render_action_cards<'a>(model: &HomeViewModel<'a>) -> Element<'a, HomeMessage> {
    let search_card = render_action_card(ActionCardContent {
        card_icon: Icon::ScanText,
        badge_kind: BadgeKind::Accent,
        title: SEARCH_CARD_TITLE,
        description: SEARCH_CARD_DESCRIPTION,
        accelerator: model.capture_hotkey,
        on_press: HomeMessage::StartCapture,
    });
    let (record_title, record_description, record_icon) = match model.is_recording {
        true => (
            STOP_RECORDING_CARD_TITLE,
            STOP_RECORDING_CARD_DESCRIPTION,
            Icon::Stop,
        ),
        false => (RECORD_CARD_TITLE, RECORD_CARD_DESCRIPTION, Icon::Record),
    };
    let record_card = render_action_card(ActionCardContent {
        card_icon: record_icon,
        badge_kind: BadgeKind::Record,
        title: record_title,
        description: record_description,
        accelerator: model.recording_hotkey,
        on_press: HomeMessage::StartRecording,
    });
    row![search_card, record_card]
        .spacing(spacing::LARGE)
        .into()
}

fn render_action_card<'a>(content: ActionCardContent<'a>) -> Element<'a, HomeMessage> {
    let card_body = column![
        render_icon_badge(content.card_icon, content.badge_kind),
        Space::new().height(Length::Fill),
        text(content.title).size(font_size::HEADING),
        text(content.description)
            .size(font_size::CAPTION)
            .style(surfaces::secondary_text),
        Space::new().height(Length::Fixed(spacing::SMALL)),
        shortcut_keycaps(content.accelerator, KeycapSurface::Window),
    ]
    .spacing(spacing::EXTRA_SMALL)
    .height(Length::Fill);
    button(card_body)
        .width(Length::Fill)
        .height(Length::Fixed(ACTION_CARD_HEIGHT))
        .padding(spacing::EXTRA_LARGE - 4.0)
        .style(button_style(ButtonTone::Card))
        .on_press(content.on_press)
        .into()
}

fn render_icon_badge<'a>(badge_icon: Icon, badge_kind: BadgeKind) -> Element<'a, HomeMessage> {
    container(icon(
        badge_icon,
        icon_size::LARGE,
        IconTint::Fixed(iced::Color::WHITE),
    ))
    .width(Length::Fixed(ICON_BADGE_SIZE))
    .height(Length::Fixed(ICON_BADGE_SIZE))
    .center_x(Length::Fixed(ICON_BADGE_SIZE))
    .center_y(Length::Fixed(ICON_BADGE_SIZE))
    .style(move |theme: &Theme| badge_style(tokens_for_theme(theme), badge_kind))
    .into()
}

fn badge_style(tokens: &DesignTokens, badge_kind: BadgeKind) -> container::Style {
    let background = match badge_kind {
        BadgeKind::Accent => tokens.accent,
        BadgeKind::Record => tokens.record,
    };
    container::Style {
        background: Some(iced::Background::Color(background)),
        border: iced::Border {
            radius: radius::MEDIUM.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
