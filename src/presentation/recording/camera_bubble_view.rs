use iced::widget::{column, container, image, mouse_area, text};
use iced::{Alignment, Border, Color, ContentFit, Element, Length};

use crate::core::models::CameraBubbleShape;
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{font_size, icon, icon_size, spacing, Icon, IconTint};

const RING_WIDTH: f32 = 3.0;
const ROUNDED_SQUARE_RADIUS: f32 = crate::presentation::design::tokens::radius::STANDARD;
const STARTING_LABEL: &str = "Starting camera…";
const PLACEHOLDER_MAXIMUM_TEXT_WIDTH: f32 = 0.7;

#[derive(Debug, Clone)]
pub enum CameraBubbleContent {
    Starting,
    Live(image::Handle),
    Unavailable(String),
}

#[derive(Debug, Clone)]
pub enum CameraBubbleMessage {
    DragRequested,
}

#[derive(Debug, Clone)]
pub struct CameraBubbleView {
    pub content: CameraBubbleContent,
    pub edge: f32,
    pub shape: CameraBubbleShape,
}

pub fn corner_radius_for(shape: CameraBubbleShape, edge: f32) -> f32 {
    match shape {
        CameraBubbleShape::Circle => edge / 2.0,
        CameraBubbleShape::RoundedSquare => ROUNDED_SQUARE_RADIUS,
    }
}

impl CameraBubbleView {
    pub fn render(&self) -> Element<'_, CameraBubbleMessage> {
        let outer_radius = corner_radius_for(self.shape, self.edge);
        let inner_edge = self.edge - RING_WIDTH * 2.0;
        let inner_radius = (outer_radius - RING_WIDTH).max(0.0);
        let body: Element<'_, CameraBubbleMessage> = match &self.content {
            CameraBubbleContent::Live(handle) => image(handle.clone())
                .width(Length::Fixed(inner_edge))
                .height(Length::Fixed(inner_edge))
                .content_fit(ContentFit::Cover)
                .border_radius(inner_radius)
                .into(),
            CameraBubbleContent::Starting => {
                self.render_placeholder(Icon::Camera, STARTING_LABEL, inner_edge)
            }
            CameraBubbleContent::Unavailable(reason) => {
                self.render_placeholder(Icon::CameraOff, reason, inner_edge)
            }
        };
        let ring_color = match self.content {
            CameraBubbleContent::Unavailable(_) => HUD_TOKENS.danger,
            _ => Color::WHITE,
        };
        let framed = container(body)
            .width(Length::Fixed(self.edge))
            .height(Length::Fixed(self.edge))
            .padding(RING_WIDTH)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(HUD_TOKENS.surface)),
                border: Border {
                    color: ring_color,
                    width: RING_WIDTH,
                    radius: outer_radius.into(),
                },
                ..Default::default()
            });
        mouse_area(framed)
            .on_press(CameraBubbleMessage::DragRequested)
            .interaction(iced::mouse::Interaction::Grab)
            .into()
    }

    fn render_placeholder<'a>(
        &self,
        placeholder_icon: Icon,
        label: &'a str,
        inner_edge: f32,
    ) -> Element<'a, CameraBubbleMessage> {
        let content = column![
            icon(
                placeholder_icon,
                icon_size::HERO,
                IconTint::Fixed(HUD_TOKENS.text_secondary)
            ),
            text(label)
                .size(font_size::SMALL)
                .width(Length::Fixed(inner_edge * PLACEHOLDER_MAXIMUM_TEXT_WIDTH))
                .align_x(Alignment::Center)
                .style(crate::presentation::design::surfaces::hud_secondary_text),
        ]
        .spacing(spacing::SMALL)
        .align_x(Alignment::Center);
        container(content)
            .width(Length::Fixed(inner_edge))
            .height(Length::Fixed(inner_edge))
            .center_x(Length::Fixed(inner_edge))
            .center_y(Length::Fixed(inner_edge))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corner_radius_matches_shape() {
        assert_eq!(corner_radius_for(CameraBubbleShape::Circle, 200.0), 100.0);
        assert_eq!(
            corner_radius_for(CameraBubbleShape::RoundedSquare, 200.0),
            ROUNDED_SQUARE_RADIUS
        );
    }
}
