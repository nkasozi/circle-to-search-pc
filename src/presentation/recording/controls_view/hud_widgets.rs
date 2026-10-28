use iced::widget::{button, container, mouse_area, row, text, tooltip, Space};
use iced::{Alignment, Element, Length};

use super::RecordingControlsMessage;
use crate::presentation::design::components::{status_dot, with_tooltip};
use crate::presentation::design::tokens::{vibrant, HUD_TOKENS};
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const DRAG_TOOLTIP: &str = "Drag to move";
const DIVIDER_HEIGHT: f32 = 24.0;

pub(super) fn hud_button(
    button_icon: Icon,
    message: RecordingControlsMessage,
    tone: ButtonTone,
    tint: IconTint,
    label: &'static str,
) -> Element<'static, RecordingControlsMessage> {
    with_tooltip(
        button(icon(button_icon, icon_size::SMALL + 2.0, tint))
            .padding(spacing::SMALL)
            .style(button_style(tone))
            .on_press(message),
        label,
        tooltip::Position::Bottom,
    )
}

pub(super) fn render_grip() -> Element<'static, RecordingControlsMessage> {
    with_tooltip(
        mouse_area(
            container(icon(
                Icon::Grip,
                icon_size::MEDIUM,
                IconTint::Fixed(HUD_TOKENS.text_muted),
            ))
            .padding([spacing::EXTRA_SMALL, 0.0]),
        )
        .on_press(RecordingControlsMessage::DragRequested)
        .interaction(iced::mouse::Interaction::Grab),
        DRAG_TOOLTIP,
        tooltip::Position::Bottom,
    )
}

pub(super) fn render_status_row(label: &'static str) -> Element<'static, RecordingControlsMessage> {
    row![
        render_grip(),
        status_dot(vibrant::YELLOW),
        text(label).size(font_size::BODY).style(surfaces::hud_text)
    ]
    .spacing(spacing::SMALL)
    .padding([spacing::SMALL, spacing::SMALL])
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn render_divider() -> Element<'static, RecordingControlsMessage> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(DIVIDER_HEIGHT))
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(HUD_TOKENS.border_strong)),
            ..Default::default()
        })
        .into()
}
