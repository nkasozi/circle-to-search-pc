use super::*;
use crate::presentation::design::components::{hud_icon_tint, with_tooltip};
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};
use crate::presentation::recording::annotation::ANNOTATION_PALETTE;

const DRAW_DISABLE_TOOLTIP: &str = "Stop drawing";
const DRAW_ENABLE_TOOLTIP: &str = "Draw on the image";
const CLEAR_DRAWINGS_TOOLTIP: &str = "Clear drawings";
const DRAG_HANDLE_LABEL: &str = "Markup";
const DRAW_PANEL_WIDTH: f32 = 300.0;
const DRAW_HANDLE_HEIGHT: f32 = 26.0;
const SWATCH_SIZE: f32 = 18.0;
const ACTION_TOOLBAR_BOTTOM_MARGIN: f32 = 16.0;
const DIVIDER_HEIGHT: f32 = 24.0;
const DRAW_PANEL_SECTION_SPACING: f32 = 2.0;
const SELECTED_SWATCH_RING_WIDTH: f32 = 3.0;
const SWATCH_RING_WIDTH: f32 = 1.0;

impl InteractiveOcrView {
    pub(super) fn build_draw_toolbar(&self) -> Element<'_, InteractiveOcrMessage> {
        let panel = container(
            iced::widget::column![render_draw_handle_strip(), self.render_draw_body()]
                .spacing(DRAW_PANEL_SECTION_SPACING),
        )
        .width(Length::Fixed(DRAW_PANEL_WIDTH))
        .padding([spacing::EXTRA_SMALL, spacing::SMALL])
        .style(surfaces::hud_panel);
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: self.draw_panel_position.y,
                left: self.draw_panel_position.x,
                right: 0.0,
                bottom: 0.0,
            })
            .align_x(Alignment::Start)
            .align_y(Alignment::Start)
            .into()
    }

    fn render_draw_body(&self) -> iced::widget::Row<'_, InteractiveOcrMessage> {
        let body = row![self.render_draw_toggle()]
            .spacing(spacing::EXTRA_SMALL)
            .align_y(Alignment::Center);
        if !self.draw_mode_enabled {
            return body;
        }
        ANNOTATION_PALETTE
            .into_iter()
            .fold(body, |body, color| body.push(self.render_swatch(color)))
            .push(render_clear_drawings_button())
    }

    fn render_draw_toggle(&self) -> Element<'_, InteractiveOcrMessage> {
        let (tone, tint, label) = match self.draw_mode_enabled {
            true => (
                ButtonTone::HudIconActive,
                IconTint::Fixed(HUD_TOKENS.accent_hover),
                DRAW_DISABLE_TOOLTIP,
            ),
            false => (ButtonTone::HudIcon, hud_icon_tint(), DRAW_ENABLE_TOOLTIP),
        };
        with_tooltip(
            button(icon(Icon::Pen, icon_size::SMALL + 2.0, tint))
                .padding(spacing::SMALL)
                .style(button_style(tone))
                .on_press(InteractiveOcrMessage::ToggleDrawMode),
            label,
            tooltip::Position::Bottom,
        )
    }

    fn render_swatch(&self, color: Color) -> Element<'static, InteractiveOcrMessage> {
        let is_selected = (self.draw_color.r - color.r).abs() < 0.01
            && (self.draw_color.g - color.g).abs() < 0.01
            && (self.draw_color.b - color.b).abs() < 0.01;
        let (ring_color, ring_width) = match is_selected {
            true => (Color::WHITE, SELECTED_SWATCH_RING_WIDTH),
            false => (HUD_TOKENS.border_strong, SWATCH_RING_WIDTH),
        };
        let swatch = container(iced::widget::Space::new())
            .width(Length::Fixed(SWATCH_SIZE))
            .height(Length::Fixed(SWATCH_SIZE))
            .style(move |_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(color)),
                border: Border {
                    color: ring_color,
                    width: ring_width,
                    radius: crate::presentation::design::tokens::radius::STANDARD.into(),
                },
                ..Default::default()
            });
        button(swatch)
            .padding(3.0)
            .style(|_theme, _status| iced::widget::button::Style {
                background: None,
                ..Default::default()
            })
            .on_press(InteractiveOcrMessage::SetDrawColor(color))
            .into()
    }

    pub(super) fn build_action_toolbar(&self) -> Element<'_, InteractiveOcrMessage> {
        let mut action_row = row![]
            .spacing(spacing::EXTRA_SMALL + 2.0)
            .align_y(Alignment::Center);
        action_row = self.push_copy_text_button(action_row);
        action_row = self.push_search_controls(action_row);
        action_row = action_row.push(render_divider());
        action_row = self.push_copy_image_button(action_row);
        action_row = self.push_save_button(action_row);
        action_row = self.push_recrop_button(action_row);
        action_row = self.push_close_button(action_row);
        container(
            container(action_row)
                .padding([spacing::EXTRA_SMALL + 2.0, spacing::SMALL + 2.0])
                .style(surfaces::hud_pill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            bottom: ACTION_TOOLBAR_BOTTOM_MARGIN,
            ..iced::Padding::ZERO
        })
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .into()
    }
}

fn render_draw_handle_strip<'a>() -> Element<'a, InteractiveOcrMessage> {
    container(
        row![
            icon(
                Icon::Grip,
                icon_size::SMALL,
                IconTint::Fixed(HUD_TOKENS.text_muted)
            ),
            text(DRAG_HANDLE_LABEL)
                .size(font_size::SMALL)
                .style(surfaces::hud_secondary_text)
        ]
        .spacing(spacing::EXTRA_SMALL)
        .align_y(Alignment::Center),
    )
    .height(Length::Fixed(DRAW_HANDLE_HEIGHT))
    .padding([0.0, spacing::SMALL])
    .center_y(Length::Fixed(DRAW_HANDLE_HEIGHT))
    .into()
}

fn render_clear_drawings_button<'a>() -> Element<'a, InteractiveOcrMessage> {
    with_tooltip(
        button(icon(Icon::Trash, icon_size::SMALL + 2.0, hud_icon_tint()))
            .padding(spacing::SMALL)
            .style(button_style(ButtonTone::HudIcon))
            .on_press(InteractiveOcrMessage::ClearDrawings),
        CLEAR_DRAWINGS_TOOLTIP,
        tooltip::Position::Bottom,
    )
}

fn render_divider() -> Element<'static, InteractiveOcrMessage> {
    container(iced::widget::Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(DIVIDER_HEIGHT))
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(HUD_TOKENS.border_strong)),
            ..Default::default()
        })
        .into()
}
