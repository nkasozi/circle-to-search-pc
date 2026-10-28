use super::styles::ActionVisualState;
use super::*;
use crate::presentation::design::components::{shortcut_keycaps, with_tooltip, KeycapSurface};
use crate::presentation::design::tokens::{radius, HUD_TOKENS};
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const COPY_TEXT_TOOLTIP_MACOS: &str = "Copy selected text (⌘C)";
const COPY_TEXT_TOOLTIP_OTHER: &str = "Copy selected text (Ctrl+C)";
const SEARCH_INPUT_PLACEHOLDER: &str = "Add words to refine…";
const SEARCH_BUTTON_LABEL: &str = "Search";
const SEARCH_BUTTON_TOOLTIP: &str = "Search this image with Google Lens";
const HELP_HINT_TEXT: &str = "Drag across text to select it";
const SELECT_ALL_SHORTCUT_MACOS: &str = "Cmd+A";
const SELECT_ALL_SHORTCUT_OTHER: &str = "Ctrl+A";
const SELECT_ALL_SUFFIX: &str = "selects everything";
const SEPARATOR_DOT: &str = "·";
const SEARCH_INPUT_WIDTH: f32 = 190.0;

fn hud_field_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => HUD_TOKENS.accent,
        _ => HUD_TOKENS.border_strong,
    };
    text_input::Style {
        background: iced::Background::Color(HUD_TOKENS.surface_raised),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::PILL.into(),
        },
        icon: HUD_TOKENS.text_muted,
        placeholder: HUD_TOKENS.text_muted,
        value: HUD_TOKENS.text_primary,
        selection: Color {
            a: 0.4,
            ..HUD_TOKENS.accent
        },
    }
}

impl InteractiveOcrView {
    pub(super) fn push_copy_text_button<'a>(
        &self,
        action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        if self.selected_chars.is_empty() {
            return action_row;
        }
        let label = match cfg!(target_os = "macos") {
            true => COPY_TEXT_TOOLTIP_MACOS,
            false => COPY_TEXT_TOOLTIP_OTHER,
        };
        let visual_state = match self.copy_state {
            CopyState::Success => ActionVisualState::Succeeded,
            CopyState::Failed => ActionVisualState::Failed,
            CopyState::Idle => ActionVisualState::Idle,
        };
        action_row.push(self.action_button(
            Icon::Copy,
            visual_state,
            InteractiveOcrMessage::CopySelected,
            label.to_string(),
        ))
    }

    pub(super) fn push_search_controls<'a>(
        &'a self,
        action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let is_searching = !matches!(self.search_state, SearchState::Idle);
        let search_input = text_input(SEARCH_INPUT_PLACEHOLDER, &self.search_query)
            .on_input(InteractiveOcrMessage::SearchQueryChanged)
            .on_submit(InteractiveOcrMessage::SearchSelected)
            .padding([spacing::SMALL, spacing::MEDIUM + 2.0])
            .size(font_size::BODY)
            .width(Length::Fixed(SEARCH_INPUT_WIDTH))
            .style(hud_field_style);
        let button_content: Element<'a, InteractiveOcrMessage> = match &self.search_state {
            SearchState::UploadingImage => text(Self::spinner_frame_text(self.spinner_frame))
                .size(font_size::HEADING)
                .into(),
            _ => row![
                icon(Icon::Search, icon_size::SMALL + 2.0, iced::Color::WHITE),
                text(SEARCH_BUTTON_LABEL).size(font_size::BODY)
            ]
            .spacing(spacing::SMALL)
            .align_y(Alignment::Center)
            .into(),
        };
        let search_button = button(button_content)
            .padding([spacing::SMALL + 1.0, spacing::LARGE])
            .style(button_style(ButtonTone::Primary))
            .on_press_maybe((!is_searching).then_some(InteractiveOcrMessage::SearchSelected));
        action_row.push(search_input).push(with_tooltip(
            search_button,
            SEARCH_BUTTON_TOOLTIP,
            tooltip::Position::Top,
        ))
    }

    pub(super) fn build_help_hint(&self) -> Element<'_, InteractiveOcrMessage> {
        let select_all_shortcut = match cfg!(target_os = "macos") {
            true => SELECT_ALL_SHORTCUT_MACOS,
            false => SELECT_ALL_SHORTCUT_OTHER,
        };
        let hint_content = row![
            text(HELP_HINT_TEXT)
                .size(font_size::CAPTION)
                .style(surfaces::hud_text),
            text(SEPARATOR_DOT)
                .size(font_size::CAPTION)
                .style(surfaces::hud_secondary_text),
            shortcut_keycaps(select_all_shortcut, KeycapSurface::Hud),
            text(SELECT_ALL_SUFFIX)
                .size(font_size::CAPTION)
                .style(surfaces::hud_secondary_text),
            button(icon(
                Icon::Close,
                icon_size::SMALL,
                IconTint::Fixed(HUD_TOKENS.text_secondary)
            ))
            .padding(spacing::EXTRA_SMALL)
            .style(button_style(ButtonTone::HudIcon))
            .on_press(InteractiveOcrMessage::DismissHelpHint),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center);
        container(hint_content)
            .padding([spacing::EXTRA_SMALL + 2.0, spacing::MEDIUM])
            .style(surfaces::hud_pill)
            .into()
    }
}
