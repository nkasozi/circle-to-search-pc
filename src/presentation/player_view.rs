use std::time::Duration;

use iced::widget::{button, column, container, image, row, slider, text, tooltip, Space};
use iced::{Alignment, Color, ContentFit, Element, Length};

use crate::core::interfaces::ports::PlaybackStatus;
use crate::core::models::format_elapsed_clock;
use crate::presentation::design::components::{hud_icon_tint, with_tooltip};
use crate::presentation::design::inputs::hud_slider_style;
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const PLAY_TOOLTIP: &str = "Play (Space)";
const PAUSE_TOOLTIP: &str = "Pause (Space)";
const REVEAL_TOOLTIP: &str = "Show in folder";
const CLOSE_TOOLTIP: &str = "Close (Esc)";
const LOADING_LABEL: &str = "Loading…";
const TIME_LABEL_WIDTH: f32 = 56.0;
const SLIDER_STEP_SECONDS: f32 = 0.01;
const VIDEO_BACKDROP: Color = Color::from_rgb(0.02, 0.02, 0.03);

#[derive(Debug, Clone)]
pub enum PlayerMessage {
    TogglePlay,
    Scrub(f32),
    ScrubReleased,
    Reveal,
    Close,
}

#[derive(Debug, Clone)]
pub struct PlayerView {
    pub file_name: String,
    pub frame: Option<image::Allocation>,
    pub status: PlaybackStatus,
    pub scrub_seconds: Option<f32>,
    pub error: Option<String>,
}

impl PlayerView {
    pub fn build(file_name: String) -> Self {
        Self {
            file_name,
            frame: None,
            status: PlaybackStatus::default(),
            scrub_seconds: None,
            error: None,
        }
    }

    pub fn displayed_position(&self) -> Duration {
        self.scrub_seconds
            .map(Duration::from_secs_f32)
            .unwrap_or(self.status.position)
    }

    pub fn render(&self) -> Element<'_, PlayerMessage> {
        let video: Element<'_, PlayerMessage> = match (&self.frame, &self.error) {
            (_, Some(error)) => text(error.as_str())
                .size(font_size::BODY)
                .style(surfaces::hud_secondary_text)
                .into(),
            (Some(allocation), None) => image(allocation.handle().clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .into(),
            (None, None) => text(LOADING_LABEL)
                .size(font_size::BODY)
                .style(surfaces::hud_secondary_text)
                .into(),
        };
        let video_area = container(video)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(VIDEO_BACKDROP)),
                ..Default::default()
            });
        column![self.render_header(), video_area, self.render_controls()]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_header(&self) -> Element<'_, PlayerMessage> {
        let header = row![
            icon(
                Icon::Film,
                icon_size::MEDIUM,
                IconTint::Fixed(HUD_TOKENS.accent_hover)
            ),
            text(self.file_name.as_str())
                .size(font_size::BODY)
                .style(surfaces::hud_text),
            Space::new().width(Length::Fill),
            control_button(Icon::Folder, PlayerMessage::Reveal, REVEAL_TOOLTIP),
            control_button(Icon::Close, PlayerMessage::Close, CLOSE_TOOLTIP),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center);
        container(header)
            .padding([spacing::SMALL, spacing::LARGE])
            .width(Length::Fill)
            .style(surfaces::hud_panel)
            .into()
    }

    fn render_controls(&self) -> Element<'_, PlayerMessage> {
        let (play_icon, play_tooltip) = match self.status.is_playing {
            true => (Icon::Pause, PAUSE_TOOLTIP),
            false => (Icon::Play, PLAY_TOOLTIP),
        };
        let duration_seconds = self.status.duration.as_secs_f32().max(SLIDER_STEP_SECONDS);
        let position_seconds = self
            .displayed_position()
            .as_secs_f32()
            .min(duration_seconds);
        let play_button = button(icon(play_icon, icon_size::MEDIUM, Color::WHITE))
            .padding(spacing::SMALL + 2.0)
            .style(button_style(ButtonTone::Primary))
            .on_press(PlayerMessage::TogglePlay);
        let controls = row![
            with_tooltip(play_button, play_tooltip, tooltip::Position::Top),
            text(format_elapsed_clock(self.displayed_position()))
                .size(font_size::CAPTION)
                .width(Length::Fixed(TIME_LABEL_WIDTH))
                .style(surfaces::hud_text),
            slider(
                0.0..=duration_seconds,
                position_seconds,
                PlayerMessage::Scrub
            )
            .step(SLIDER_STEP_SECONDS)
            .on_release(PlayerMessage::ScrubReleased)
            .style(hud_slider_style),
            text(format_elapsed_clock(self.status.duration))
                .size(font_size::CAPTION)
                .width(Length::Fixed(TIME_LABEL_WIDTH))
                .style(surfaces::hud_secondary_text),
        ]
        .spacing(spacing::MEDIUM)
        .align_y(Alignment::Center);
        container(controls)
            .padding([spacing::MEDIUM, spacing::LARGE])
            .width(Length::Fill)
            .style(surfaces::hud_panel)
            .into()
    }
}

fn control_button(
    button_icon: Icon,
    message: PlayerMessage,
    label: &'static str,
) -> Element<'static, PlayerMessage> {
    with_tooltip(
        button(icon(button_icon, icon_size::MEDIUM, hud_icon_tint()))
            .padding(spacing::SMALL)
            .style(button_style(ButtonTone::HudIcon))
            .on_press(message),
        label,
        tooltip::Position::Bottom,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_displayed_position_prefers_scrub_value() {
        let mut view = PlayerView::build("clip.mp4".into());
        view.status.position = Duration::from_secs(3);
        assert_eq!(view.displayed_position(), Duration::from_secs(3));
        view.scrub_seconds = Some(7.5);
        assert_eq!(view.displayed_position(), Duration::from_secs_f32(7.5));
    }
}
