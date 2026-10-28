use iced::widget::{canvas, column, container, row, stack, text};
use iced::{Alignment, Element, Length, Padding};

use super::panel::render_panel;
use super::{RecordingSetupMessage, RecordingSetupView};
use crate::core::models::CaptureMode;
use crate::presentation::design::components::{shortcut_keycaps, KeycapSurface};
use crate::presentation::design::{font_size, spacing, surfaces};

const HINT_DRAG_TO_SELECT: &str = "Drag to select the area you want to record";
const HINT_ADJUST_SELECTION: &str = "Drag again to adjust, or press Record";
const HINT_FULL_SCREEN: &str = "The whole screen will be recorded";
const HINT_CANCEL_SUFFIX: &str = "to cancel";
const ESCAPE_KEY_LABEL: &str = "Esc";
const HINT_SEPARATOR: &str = "·";
const PANEL_BOTTOM_MARGIN: f32 = 48.0;
const HINT_TOP_MARGIN: f32 = 64.0;

impl RecordingSetupView {
    pub fn render(&self) -> Element<'_, RecordingSetupMessage> {
        let selection_canvas = canvas(self).width(Length::Fill).height(Length::Fill);
        let mut layers: Vec<Element<'_, RecordingSetupMessage>> = vec![selection_canvas.into()];
        let is_interactive = self.countdown_remaining.is_none() && self.drag_origin.is_none();
        if is_interactive {
            layers.push(self.render_hint());
            layers.push(self.render_panel_layer());
        }
        stack(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn hint_text(&self) -> &str {
        match (&self.notice, self.mode, self.selected_region) {
            (Some(notice), _, _) => notice.as_str(),
            (None, CaptureMode::FullScreen, _) => HINT_FULL_SCREEN,
            (None, CaptureMode::Region, Some(_)) => HINT_ADJUST_SELECTION,
            (None, CaptureMode::Region, None) => HINT_DRAG_TO_SELECT,
        }
    }

    fn render_hint(&self) -> Element<'_, RecordingSetupMessage> {
        let hint_row = row![
            text(self.hint_text())
                .size(font_size::BODY)
                .style(surfaces::hud_text),
            text(HINT_SEPARATOR)
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

    fn render_panel_layer(&self) -> Element<'_, RecordingSetupMessage> {
        container(column![render_panel(self)])
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
}
