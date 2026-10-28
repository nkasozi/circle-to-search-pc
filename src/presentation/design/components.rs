use iced::widget::{button, container, row, text, tooltip, Button, Row, Space};
use iced::{Alignment, Color, Element, Length, Theme};

use super::buttons::{button_style, ButtonTone};
use super::icons::{icon, Icon, IconTint};
use super::surfaces;
use super::tokens::{font_size, icon_size, spacing, HUD_TOKENS};
use crate::core::models::{current_platform_key_symbol_style, KeySymbolStyle};

const ACCELERATOR_SEPARATOR: char = '+';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeycapSurface {
    Window,
    Hud,
}

pub fn icon_button<'a, Message: Clone + 'a>(
    button_icon: Icon,
    on_press: Message,
    tone: ButtonTone,
    icon_tint: impl Into<IconTint>,
) -> Button<'a, Message> {
    button(icon(button_icon, icon_size::MEDIUM, icon_tint))
        .padding(spacing::SMALL + 1.0)
        .style(button_style(tone))
        .on_press(on_press)
}

pub fn labeled_button<'a, Message: Clone + 'a>(
    button_icon: Icon,
    label: &'a str,
    on_press: Message,
    tone: ButtonTone,
    content_tint: impl Into<IconTint>,
) -> Button<'a, Message> {
    let content = row![
        icon(button_icon, icon_size::SMALL + 2.0, content_tint),
        text(label).size(font_size::BODY)
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    button(content)
        .padding([spacing::SMALL + 2.0, spacing::LARGE])
        .style(button_style(tone))
        .on_press(on_press)
}

pub fn with_tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl iced::widget::text::IntoFragment<'a>,
    position: tooltip::Position,
) -> Element<'a, Message> {
    let tooltip_body = container(
        text(label)
            .size(font_size::CAPTION)
            .style(surfaces::hud_text),
    )
    .padding([spacing::EXTRA_SMALL + 2.0, spacing::SMALL + 2.0])
    .style(surfaces::tooltip);
    tooltip(content, tooltip_body, position)
        .gap(spacing::SMALL)
        .into()
}

pub fn shortcut_keycaps<'a, Message: 'a>(
    accelerator: &str,
    surface: KeycapSurface,
) -> Row<'a, Message> {
    let symbol_style = current_platform_key_symbol_style();
    let keycaps: Vec<Element<'a, Message>> = accelerator
        .split(ACCELERATOR_SEPARATOR)
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| render_keycap(token, symbol_style, surface))
        .collect();
    Row::with_children(keycaps)
        .spacing(spacing::EXTRA_SMALL)
        .align_y(Alignment::Center)
}

fn render_keycap<'a, Message: 'a>(
    token: &str,
    symbol_style: KeySymbolStyle,
    surface: KeycapSurface,
) -> Element<'a, Message> {
    let label = crate::core::models::format_accelerator_for_display(token, symbol_style);
    let keycap_style = match surface {
        KeycapSurface::Window => surfaces::keycap,
        KeycapSurface::Hud => surfaces::hud_keycap,
    };
    container(text(label).size(font_size::CAPTION))
        .padding([2.0, spacing::SMALL - 1.0])
        .style(keycap_style)
        .into()
}

pub fn status_dot<'a, Message: 'a>(color: Color) -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .style(move |_theme: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub fn hud_icon_tint() -> IconTint {
    IconTint::Fixed(HUD_TOKENS.text_primary)
}
