use iced::widget::{column, container, row, text, Column};
use iced::{Alignment, Element, Length, Theme};

use super::constants::{FEATURE_BADGE_SIZE, HERO_BADGE_SIZE};
use super::OnboardingMessage;
use crate::presentation::design::components::{shortcut_keycaps, KeycapSurface};
use crate::presentation::design::tokens::radius;
use crate::presentation::design::{font_size, icon, icon_size, spacing, surfaces, Icon};

pub(super) fn hero_badge<'a>(
    badge_icon: Icon,
    color: iced::Color,
) -> Element<'a, OnboardingMessage> {
    badge(badge_icon, color, HERO_BADGE_SIZE, icon_size::HERO)
}

pub(super) fn feature_badge<'a>(
    badge_icon: Icon,
    color: iced::Color,
) -> Element<'a, OnboardingMessage> {
    badge(badge_icon, color, FEATURE_BADGE_SIZE, icon_size::MEDIUM)
}

fn badge<'a>(
    badge_icon: Icon,
    color: iced::Color,
    size: f32,
    glyph_size: f32,
) -> Element<'a, OnboardingMessage> {
    container(icon(badge_icon, glyph_size, iced::Color::WHITE))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .style(move |_theme: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                radius: radius::MEDIUM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub(super) fn heading<'a>(title: &'a str, subtitle: &'a str) -> Column<'a, OnboardingMessage> {
    column![
        text(title).size(font_size::DISPLAY),
        text(subtitle)
            .size(font_size::BODY)
            .style(surfaces::secondary_text),
    ]
    .spacing(spacing::SMALL)
}

pub(super) fn feature_row<'a>(
    feature_icon: Icon,
    color: iced::Color,
    title: &'a str,
    description: &'a str,
) -> Element<'a, OnboardingMessage> {
    row![
        feature_badge(feature_icon, color),
        column![
            text(title).size(font_size::HEADING),
            text(description)
                .size(font_size::CAPTION)
                .style(surfaces::secondary_text)
        ]
        .spacing(2.0),
    ]
    .spacing(spacing::LARGE)
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn shortcut_row<'a>(
    label: &'static str,
    accelerator: &'a str,
    shortcut_icon: Icon,
    color: iced::Color,
) -> Element<'a, OnboardingMessage> {
    container(
        row![
            feature_badge(shortcut_icon, color),
            text(label).size(font_size::HEADING).width(Length::Fill),
            shortcut_keycaps(accelerator, KeycapSurface::Window),
        ]
        .spacing(spacing::LARGE)
        .align_y(Alignment::Center),
    )
    .padding(spacing::LARGE)
    .style(surfaces::card)
    .into()
}
