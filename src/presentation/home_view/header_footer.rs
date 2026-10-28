use iced::widget::{column, container, row, text, toggler, Space};
use iced::{Alignment, Element, Length, Theme};

use super::{HomeMessage, HomeViewModel, StatusTone};
use crate::global_constants;
use crate::presentation::design::components::{icon_button, status_dot, with_tooltip};
use crate::presentation::design::inputs::switch_style;
use crate::presentation::design::tokens::{radius, tokens_for_theme, DARK_TOKENS};
use crate::presentation::design::{
    font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const LOGO_MARK_SIZE: f32 = 40.0;
const TITLE_SPACING: f32 = 2.0;
const FOOTER_SWITCH_SIZE: f32 = 18.0;
const HOME_SUBTITLE: &str = "Search, capture and record anything on your screen";
const SETTINGS_TOOLTIP: &str = "Settings";
const SHOW_ON_LAUNCH_LABEL: &str = "Show this window when the app starts";

pub(super) fn render_header<'a>() -> Element<'a, HomeMessage> {
    let titles = column![
        text(global_constants::APPLICATION_TITLE).size(font_size::TITLE),
        text(HOME_SUBTITLE)
            .size(font_size::CAPTION)
            .style(surfaces::secondary_text),
    ]
    .spacing(TITLE_SPACING);
    let settings_button = with_tooltip(
        icon_button(
            Icon::Sliders,
            HomeMessage::OpenSettings,
            ButtonTone::Ghost,
            IconTint::TextSecondary,
        ),
        SETTINGS_TOOLTIP,
        iced::widget::tooltip::Position::Bottom,
    );
    row![
        render_logo_mark(),
        titles,
        Space::new().width(Length::Fill),
        settings_button
    ]
    .spacing(spacing::MEDIUM)
    .align_y(Alignment::Center)
    .into()
}

fn render_logo_mark<'a>() -> Element<'a, HomeMessage> {
    container(icon(
        Icon::ScanText,
        icon_size::LARGE,
        DARK_TOKENS.on_accent,
    ))
    .width(Length::Fixed(LOGO_MARK_SIZE))
    .height(Length::Fixed(LOGO_MARK_SIZE))
    .center_x(Length::Fixed(LOGO_MARK_SIZE))
    .center_y(Length::Fixed(LOGO_MARK_SIZE))
    .style(|theme: &Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(tokens_for_theme(theme).accent)),
        border: iced::Border {
            radius: radius::MEDIUM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

pub(super) fn render_footer<'a>(model: &HomeViewModel<'a>) -> Element<'a, HomeMessage> {
    let status_tone = model.status_tone;
    let status_row = row![
        status_dot_for_tone(status_tone),
        text(model.status_text)
            .size(font_size::CAPTION)
            .style(surfaces::secondary_text),
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    let keep_running_switch = toggler(model.show_home_on_launch)
        .label(SHOW_ON_LAUNCH_LABEL)
        .text_size(font_size::CAPTION)
        .size(FOOTER_SWITCH_SIZE)
        .style(switch_style)
        .on_toggle(HomeMessage::ToggleShowHomeOnLaunch);
    row![
        status_row,
        Space::new().width(Length::Fill),
        keep_running_switch
    ]
    .align_y(Alignment::Center)
    .into()
}

fn status_dot_for_tone<'a>(tone: StatusTone) -> Element<'a, HomeMessage> {
    let color = match tone {
        StatusTone::Neutral => DARK_TOKENS.text_muted,
        StatusTone::Busy => DARK_TOKENS.warning,
        StatusTone::Success => DARK_TOKENS.success,
        StatusTone::Error => DARK_TOKENS.danger,
    };
    status_dot(color)
}
