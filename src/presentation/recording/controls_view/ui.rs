use std::time::Duration;

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use super::drawing_tools::render_drawing_tools;
use super::hud_widgets::{hud_button, render_divider, render_grip, render_status_row};
use super::{ControlsPhase, RecordingControlsMessage, RecordingControlsView};
use crate::core::models::format_elapsed_clock;
use crate::presentation::design::components::{hud_icon_tint, status_dot};
use crate::presentation::design::tokens::{vibrant, HUD_TOKENS};
use crate::presentation::design::{font_size, spacing, surfaces, ButtonTone, Icon, IconTint};

const PAUSE_TOOLTIP: &str = "Pause";
const RESUME_TOOLTIP: &str = "Resume";
const STOP_TOOLTIP: &str = "Stop and save";
const DRAW_TOOLTIP: &str = "Draw on screen";
const DISCARD_TOOLTIP: &str = "Discard recording";
const DISCARD_CONFIRM_TOOLTIP: &str = "Click again to discard";
const SAVING_LABEL: &str = "Saving your recording…";
const STARTING_LABEL: &str = "Starting…";
const PAUSED_LABEL: &str = "Paused";
const TIMER_WIDTH: f32 = 62.0;

impl RecordingControlsView {
    pub fn render(&self) -> Element<'_, RecordingControlsMessage> {
        let main_row: Element<'_, RecordingControlsMessage> = match self.phase {
            ControlsPhase::Saving => render_status_row(SAVING_LABEL),
            ControlsPhase::Starting => render_status_row(STARTING_LABEL),
            ControlsPhase::Recording { .. } => self.render_recording_row(),
        };
        let mut sections = column![container(main_row)
            .padding([spacing::EXTRA_SMALL + 2.0, spacing::SMALL + 2.0])
            .style(surfaces::hud_pill)]
        .spacing(spacing::SMALL)
        .align_x(Alignment::Center);
        if self.drawing_enabled && matches!(self.phase, ControlsPhase::Recording { .. }) {
            sections = sections.push(render_drawing_tools(self));
        }
        container(sections)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }

    fn render_recording_row(&self) -> Element<'_, RecordingControlsMessage> {
        let ControlsPhase::Recording { elapsed, is_paused } = self.phase else {
            return Space::new().into();
        };
        let (pause_icon, pause_tooltip) = match is_paused {
            true => (Icon::Play, RESUME_TOOLTIP),
            false => (Icon::Pause, PAUSE_TOOLTIP),
        };
        row![
            render_grip(),
            status_dot(indicator_color(elapsed, is_paused)),
            text(timer_label(elapsed, is_paused))
                .size(font_size::BODY)
                .width(Length::Fixed(TIMER_WIDTH))
                .style(surfaces::hud_text),
            render_divider(),
            hud_button(
                pause_icon,
                RecordingControlsMessage::TogglePause,
                ButtonTone::HudIcon,
                hud_icon_tint(),
                pause_tooltip
            ),
            hud_button(
                Icon::Stop,
                RecordingControlsMessage::Stop,
                ButtonTone::HudRecord,
                IconTint::Fixed(iced::Color::WHITE),
                STOP_TOOLTIP
            ),
            render_divider(),
            self.render_draw_toggle(),
            self.render_discard_button(),
        ]
        .spacing(spacing::SMALL - 2.0)
        .align_y(Alignment::Center)
        .into()
    }

    fn render_draw_toggle(&self) -> Element<'_, RecordingControlsMessage> {
        let (tone, tint) = match self.drawing_enabled {
            true => (
                ButtonTone::HudIconActive,
                IconTint::Fixed(HUD_TOKENS.accent_hover),
            ),
            false => (ButtonTone::HudIcon, hud_icon_tint()),
        };
        hud_button(
            Icon::Pen,
            RecordingControlsMessage::ToggleDrawing,
            tone,
            tint,
            DRAW_TOOLTIP,
        )
    }

    fn render_discard_button(&self) -> Element<'_, RecordingControlsMessage> {
        let (tone, tint, label) = match self.discard_armed {
            true => (
                ButtonTone::HudRecord,
                IconTint::Fixed(iced::Color::WHITE),
                DISCARD_CONFIRM_TOOLTIP,
            ),
            false => (
                ButtonTone::HudIcon,
                IconTint::Fixed(HUD_TOKENS.text_secondary),
                DISCARD_TOOLTIP,
            ),
        };
        hud_button(
            Icon::Trash,
            RecordingControlsMessage::Discard,
            tone,
            tint,
            label,
        )
    }
}

fn indicator_color(elapsed: Duration, is_paused: bool) -> Color {
    let blink_visible = is_paused || elapsed.as_secs() % 2 == 0;
    match (is_paused, blink_visible) {
        (true, _) => vibrant::YELLOW,
        (false, true) => vibrant::RED,
        (false, false) => HUD_TOKENS.surface_hover,
    }
}

fn timer_label(elapsed: Duration, is_paused: bool) -> String {
    match is_paused {
        true => PAUSED_LABEL.to_string(),
        false => format_elapsed_clock(elapsed),
    }
}
