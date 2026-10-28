use iced::widget::{button, container, row, Space};
use iced::{Alignment, Border, Color, Element, Length};

use super::hud_widgets::hud_button;
use super::{RecordingControlsMessage, RecordingControlsView};
use crate::presentation::design::components::hud_icon_tint;
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{spacing, surfaces, ButtonTone, Icon, IconTint};
use crate::presentation::recording::annotation::{AnnotationTool, ANNOTATION_PALETTE};

const SWATCH_SIZE: f32 = 20.0;
const SELECTED_SWATCH_RING: f32 = 3.0;
const UNSELECTED_SWATCH_RING: f32 = 1.0;
const SWATCH_PADDING: f32 = 2.0;
const TOOLBAR_ITEM_SPACING: f32 = 2.0;
const PEN_TOOLTIP: &str = "Pen";
const HIGHLIGHTER_TOOLTIP: &str = "Highlighter";
const ARROW_TOOLTIP: &str = "Arrow";
const RECTANGLE_TOOLTIP: &str = "Box";
const UNDO_TOOLTIP: &str = "Undo";
const CLEAR_TOOLTIP: &str = "Clear drawings";

const TOOL_BUTTONS: [(AnnotationTool, Icon, &str); 4] = [
    (AnnotationTool::Pen, Icon::Pen, PEN_TOOLTIP),
    (
        AnnotationTool::Highlighter,
        Icon::Highlighter,
        HIGHLIGHTER_TOOLTIP,
    ),
    (AnnotationTool::Arrow, Icon::Arrow, ARROW_TOOLTIP),
    (
        AnnotationTool::Rectangle,
        Icon::Rectangle,
        RECTANGLE_TOOLTIP,
    ),
];

pub(super) fn render_drawing_tools(
    view: &RecordingControlsView,
) -> Element<'_, RecordingControlsMessage> {
    let tool_buttons: Vec<Element<'_, RecordingControlsMessage>> = TOOL_BUTTONS
        .iter()
        .map(|(tool, tool_icon, label)| {
            render_tool_button(*tool, *tool_icon, label, view.selected_tool == *tool)
        })
        .collect();
    let swatches: Vec<Element<'_, RecordingControlsMessage>> = ANNOTATION_PALETTE
        .iter()
        .map(|color| render_swatch(*color, *color == view.selected_color))
        .collect();
    let content = row![
        iced::widget::Row::with_children(tool_buttons).spacing(TOOLBAR_ITEM_SPACING),
        Space::new().width(Length::Fixed(spacing::SMALL)),
        iced::widget::Row::with_children(swatches)
            .spacing(spacing::SMALL - 2.0)
            .align_y(Alignment::Center),
        Space::new().width(Length::Fixed(spacing::SMALL)),
        render_undo_button(view.has_drawings),
        hud_button(
            Icon::Trash,
            RecordingControlsMessage::ClearDrawings,
            ButtonTone::HudIcon,
            hud_icon_tint(),
            CLEAR_TOOLTIP
        ),
    ]
    .spacing(TOOLBAR_ITEM_SPACING)
    .align_y(Alignment::Center);
    container(content)
        .padding([spacing::EXTRA_SMALL, spacing::SMALL + 2.0])
        .style(surfaces::hud_pill)
        .into()
}

fn render_tool_button(
    tool: AnnotationTool,
    tool_icon: Icon,
    label: &'static str,
    is_selected: bool,
) -> Element<'static, RecordingControlsMessage> {
    let (tone, tint) = match is_selected {
        true => (
            ButtonTone::HudIconActive,
            IconTint::Fixed(HUD_TOKENS.accent_hover),
        ),
        false => (ButtonTone::HudIcon, hud_icon_tint()),
    };
    hud_button(
        tool_icon,
        RecordingControlsMessage::SelectTool(tool),
        tone,
        tint,
        label,
    )
}

fn render_undo_button(has_drawings: bool) -> Element<'static, RecordingControlsMessage> {
    match has_drawings {
        true => hud_button(
            Icon::Undo,
            RecordingControlsMessage::Undo,
            ButtonTone::HudIcon,
            hud_icon_tint(),
            UNDO_TOOLTIP,
        ),
        false => Space::new().into(),
    }
}

fn render_swatch(color: Color, is_selected: bool) -> Element<'static, RecordingControlsMessage> {
    let (ring_color, ring_width) = match is_selected {
        true => (Color::WHITE, SELECTED_SWATCH_RING),
        false => (HUD_TOKENS.border_strong, UNSELECTED_SWATCH_RING),
    };
    let swatch = container(Space::new())
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
        .padding(SWATCH_PADDING)
        .style(|_theme, _status| iced::widget::button::Style {
            background: None,
            ..Default::default()
        })
        .on_press(RecordingControlsMessage::SelectColor(color))
        .into()
}
