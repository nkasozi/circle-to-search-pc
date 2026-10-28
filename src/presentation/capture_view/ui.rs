use super::*;
use iced::widget::{button, canvas, container, image, row, stack, text, tooltip, Space};
use iced::{Alignment, ContentFit, Element, Length, Padding};

use crate::presentation::design::components::{
    hud_icon_tint, shortcut_keycaps, with_tooltip, KeycapSurface,
};
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const HINT_RECTANGLE: &str = "Drag over anything to search it";
const HINT_FREEFORM: &str = "Circle anything to search it";
const HINT_CANCEL_SUFFIX: &str = "to cancel";
const ESCAPE_KEY_LABEL: &str = "Esc";
const SEPARATOR_DOT: &str = "·";
const AREA_TOOLTIP: &str = "Drag a box";
const FREEFORM_TOOLTIP: &str = "Draw a circle";
const WINDOW_TOOLTIP: &str = "Pick a window";
const RECORD_LABEL: &str = "Record";
const RECORD_TOOLTIP: &str = "Record your screen instead";
const PANEL_BOTTOM_MARGIN: f32 = 48.0;
const HINT_TOP_MARGIN: f32 = 64.0;
const DIVIDER_HEIGHT: f32 = 28.0;

impl CaptureView {
    pub fn render_ui(&self) -> Element<'_, CaptureViewMessage> {
        let screenshot = image(self.capture_buffer.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Contain);
        let overlay_canvas = canvas(self).width(Length::Fill).height(Length::Fill);
        let mut layers: Vec<Element<'_, CaptureViewMessage>> =
            vec![screenshot.into(), overlay_canvas.into()];
        if !self.is_selecting {
            layers.push(self.render_hint());
            layers.push(self.render_toolbar());
        }
        container(stack(layers))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_hint(&self) -> Element<'_, CaptureViewMessage> {
        let hint_text = match self.draw_mode {
            DrawMode::Rectangle => HINT_RECTANGLE,
            DrawMode::Freeform => HINT_FREEFORM,
        };
        let hint_row = row![
            text(hint_text)
                .size(font_size::BODY)
                .style(surfaces::hud_text),
            text(SEPARATOR_DOT)
                .size(font_size::BODY)
                .style(surfaces::hud_secondary_text),
            shortcut_keycaps(ESCAPE_KEY_LABEL, KeycapSurface::Hud),
            text(HINT_CANCEL_SUFFIX)
                .size(font_size::BODY)
                .style(surfaces::hud_secondary_text),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center);
        container(
            container(hint_row)
                .padding([spacing::SMALL + 2.0, spacing::LARGE])
                .style(surfaces::hud_pill),
        )
        .width(Length::Fill)
        .padding(Padding {
            top: HINT_TOP_MARGIN,
            ..Padding::ZERO
        })
        .align_x(Alignment::Center)
        .into()
    }

    fn render_toolbar(&self) -> Element<'_, CaptureViewMessage> {
        let toolbar = row![
            self.render_mode_button(
                Icon::Region,
                AREA_TOOLTIP,
                CaptureViewMessage::SetDrawMode(DrawMode::Rectangle),
                self.draw_mode == DrawMode::Rectangle
            ),
            self.render_mode_button(
                Icon::Lasso,
                FREEFORM_TOOLTIP,
                CaptureViewMessage::SetDrawMode(DrawMode::Freeform),
                self.draw_mode == DrawMode::Freeform
            ),
            self.render_mode_button(
                Icon::AppWindow,
                WINDOW_TOOLTIP,
                CaptureViewMessage::SelectWindow,
                false
            ),
            render_divider(),
            render_record_button(),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center);
        container(
            container(toolbar)
                .padding(spacing::SMALL + 2.0)
                .style(surfaces::hud_panel),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            bottom: PANEL_BOTTOM_MARGIN,
            ..Padding::ZERO
        })
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .into()
    }

    fn render_mode_button(
        &self,
        mode_icon: Icon,
        label: &'static str,
        message: CaptureViewMessage,
        is_active: bool,
    ) -> Element<'static, CaptureViewMessage> {
        let (tone, tint) = match is_active {
            true => (
                ButtonTone::HudIconActive,
                IconTint::Fixed(HUD_TOKENS.accent_hover),
            ),
            false => (ButtonTone::HudIcon, hud_icon_tint()),
        };
        with_tooltip(
            button(icon(mode_icon, icon_size::MEDIUM, tint))
                .padding(spacing::SMALL + 1.0)
                .style(button_style(tone))
                .on_press(message),
            label,
            tooltip::Position::Top,
        )
    }
}

fn render_record_button() -> Element<'static, CaptureViewMessage> {
    let content = row![
        icon(Icon::Record, icon_size::SMALL + 2.0, iced::Color::WHITE),
        text(RECORD_LABEL).size(font_size::BODY)
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    with_tooltip(
        button(content)
            .padding([spacing::SMALL + 1.0, spacing::LARGE])
            .style(button_style(ButtonTone::HudRecord))
            .on_press(CaptureViewMessage::SwitchToRecording),
        RECORD_TOOLTIP,
        tooltip::Position::Top,
    )
}

fn render_divider() -> Element<'static, CaptureViewMessage> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(DIVIDER_HEIGHT))
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(HUD_TOKENS.border_strong)),
            ..Default::default()
        })
        .into()
}
