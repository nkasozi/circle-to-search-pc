use iced::widget::{column, container, row, scrollable, text, Column, Space};
use iced::{Alignment, Element, Length};

use super::controls::segmented;
use super::general_tab::render_general_tab;
use super::recording_tab::render_recording_tab;
use super::search_tab::render_search_tab;
use super::{SettingsMessage, SettingsTab, SettingsViewState};
use crate::core::models::UserSettings;
use crate::presentation::design::components::labeled_button;
use crate::presentation::design::{font_size, spacing, surfaces, ButtonTone, Icon, IconTint};

const SETTINGS_TITLE: &str = "Settings";
const GENERAL_TAB_LABEL: &str = "General";
const RECORDING_TAB_LABEL: &str = "Recording";
const SEARCH_TAB_LABEL: &str = "Search";
const SAVE_LABEL: &str = "Save";
const CANCEL_LABEL: &str = "Cancel";
const CONTENT_MAXIMUM_WIDTH: f32 = 640.0;

pub fn render_settings<'a>(
    state: &'a SettingsViewState,
    draft: &'a UserSettings,
) -> Element<'a, SettingsMessage> {
    let header = row![
        text(SETTINGS_TITLE).size(font_size::TITLE),
        Space::new().width(Length::Fill),
        segmented(
            &[
                (SettingsTab::General, GENERAL_TAB_LABEL),
                (SettingsTab::Recording, RECORDING_TAB_LABEL),
                (SettingsTab::Search, SEARCH_TAB_LABEL)
            ],
            state.tab,
            SettingsMessage::SelectTab,
        ),
    ]
    .align_y(Alignment::Center);
    let cards = match state.tab {
        SettingsTab::General => render_general_tab(state, draft),
        SettingsTab::Recording => render_recording_tab(draft),
        SettingsTab::Search => render_search_tab(draft),
    };
    let body = scrollable(
        Column::with_children(cards)
            .spacing(spacing::LARGE)
            .padding(iced::Padding {
                right: spacing::LARGE,
                ..iced::Padding::ZERO
            }),
    )
    .height(Length::Fill);
    let layout = column![header, body, render_footer(state)]
        .spacing(spacing::EXTRA_LARGE)
        .padding(spacing::EXTRA_LARGE + 4.0)
        .max_width(CONTENT_MAXIMUM_WIDTH);
    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .style(surfaces::window_background)
        .into()
}

fn render_footer(state: &SettingsViewState) -> Element<'_, SettingsMessage> {
    let error_text: Element<'_, SettingsMessage> = match &state.validation_error {
        Some(error) => text(error.as_str())
            .size(font_size::CAPTION)
            .style(surfaces::colored_text(
                crate::presentation::design::tokens::DARK_TOKENS.danger,
            ))
            .into(),
        None => Space::new().into(),
    };
    row![
        container(error_text).width(Length::Fill),
        labeled_button(
            Icon::Close,
            CANCEL_LABEL,
            SettingsMessage::Cancel,
            ButtonTone::Ghost,
            IconTint::TextSecondary
        ),
        labeled_button(
            Icon::Check,
            SAVE_LABEL,
            SettingsMessage::Save,
            ButtonTone::Primary,
            iced::Color::WHITE
        ),
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center)
    .into()
}
