use super::*;
use crate::presentation::design::components::with_tooltip;
use crate::presentation::design::surfaces::{toast, ToastKind};
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActionVisualState {
    Idle,
    Busy,
    Succeeded,
    Failed,
}

impl InteractiveOcrView {
    pub(super) fn build_toast_with_kind<'a>(
        message: String,
        kind: ToastKind,
    ) -> Element<'a, InteractiveOcrMessage> {
        let (toast_icon, tint) = match kind {
            ToastKind::Success => (Icon::Check, IconTint::Success),
            ToastKind::Error => (Icon::Close, IconTint::Danger),
        };
        container(
            row![
                icon(toast_icon, icon_size::SMALL, tint),
                text(message)
                    .size(font_size::BODY)
                    .style(surfaces::hud_text)
            ]
            .spacing(spacing::SMALL)
            .align_y(Alignment::Center),
        )
        .padding([spacing::SMALL, spacing::LARGE])
        .style(toast(kind))
        .into()
    }

    pub(super) fn render_image_with_overlay(&self) -> Element<'_, InteractiveOcrMessage> {
        let picture = image(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Contain);
        let ocr_overlay = OcrOverlay {
            char_positions: self.char_positions.clone(),
            image_width: self.image_width,
            image_height: self.image_height,
            selected_indices: self.selected_chars.clone(),
            draw_strokes: self.draw_strokes.clone(),
            current_stroke_points: self.current_stroke_points.clone(),
            is_drawing: self.is_drawing,
            draw_color: self.draw_color,
            draw_width: self.draw_width,
            draw_mode_enabled: self.draw_mode_enabled,
            draw_panel_position: self.draw_panel_position,
            draw_panel_is_dragging: self.draw_panel_is_dragging,
        };
        stack![
            picture,
            canvas(ocr_overlay).width(Length::Fill).height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub(super) fn spinner_frame_text(spinner_frame: usize) -> &'static str {
        SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()]
    }

    pub(super) fn action_button(
        &self,
        idle_icon: Icon,
        visual_state: ActionVisualState,
        message: InteractiveOcrMessage,
        label: String,
    ) -> Element<'static, InteractiveOcrMessage> {
        let content: Element<'static, InteractiveOcrMessage> = match visual_state {
            ActionVisualState::Idle => icon(
                idle_icon,
                icon_size::MEDIUM,
                IconTint::Fixed(crate::presentation::design::tokens::HUD_TOKENS.text_primary),
            )
            .into(),
            ActionVisualState::Busy => text(Self::spinner_frame_text(self.spinner_frame))
                .size(font_size::HEADING)
                .style(surfaces::hud_text)
                .into(),
            ActionVisualState::Succeeded => {
                icon(Icon::Check, icon_size::MEDIUM, IconTint::Success).into()
            }
            ActionVisualState::Failed => {
                icon(Icon::Close, icon_size::MEDIUM, IconTint::Danger).into()
            }
        };
        let action = button(
            container(content)
                .width(Length::Fixed(icon_size::MEDIUM))
                .center_x(Length::Fixed(icon_size::MEDIUM)),
        )
        .padding(spacing::SMALL + 1.0)
        .style(button_style(ButtonTone::HudIcon))
        .on_press_maybe((visual_state == ActionVisualState::Idle).then_some(message));
        with_tooltip(action, label, tooltip::Position::Top)
    }
}
