use iced::widget::{button, column, container, row, text, toggler, Row};
use iced::{Alignment, Element, Length};

use super::SettingsMessage;
use crate::presentation::design::inputs::switch_style;
use crate::presentation::design::{button_style, font_size, spacing, surfaces, ButtonTone};

pub fn segmented<T: Copy + PartialEq + 'static>(
    options: &[(T, &'static str)],
    selected: T,
    on_select: fn(T) -> SettingsMessage,
) -> Element<'static, SettingsMessage> {
    let buttons: Vec<Element<'static, SettingsMessage>> = options
        .iter()
        .map(|(value, label)| {
            let tone = match *value == selected {
                true => ButtonTone::SegmentActive,
                false => ButtonTone::Segment,
            };
            button(text(*label).size(font_size::CAPTION))
                .padding([spacing::EXTRA_SMALL + 2.0, spacing::MEDIUM])
                .style(button_style(tone))
                .on_press(on_select(*value))
                .into()
        })
        .collect();
    Row::with_children(buttons)
        .spacing(spacing::EXTRA_SMALL)
        .into()
}

pub fn setting_row<'a>(
    label: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, SettingsMessage>>,
) -> Element<'a, SettingsMessage> {
    row![
        column![
            text(label).size(font_size::BODY),
            text(description)
                .size(font_size::CAPTION)
                .style(surfaces::muted_text),
        ]
        .spacing(2.0)
        .width(Length::Fill),
        control.into(),
    ]
    .spacing(spacing::LARGE)
    .align_y(Alignment::Center)
    .into()
}

pub fn switch_row<'a>(
    label: &'a str,
    description: &'a str,
    is_on: bool,
    on_toggle: fn(bool) -> SettingsMessage,
) -> Element<'a, SettingsMessage> {
    setting_row(
        label,
        description,
        toggler(is_on)
            .size(18.0)
            .style(switch_style)
            .on_toggle(on_toggle),
    )
}

pub fn settings_card<'a>(
    title: &'a str,
    rows: Vec<Element<'a, SettingsMessage>>,
) -> Element<'a, SettingsMessage> {
    let mut body = column![text(title).size(font_size::HEADING)].spacing(spacing::LARGE);
    for content_row in rows {
        body = body.push(content_row);
    }
    container(body)
        .padding(spacing::LARGE + 2.0)
        .width(Length::Fill)
        .style(surfaces::card)
        .into()
}
