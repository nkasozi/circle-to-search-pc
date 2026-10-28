use std::time::SystemTime;

use iced::widget::{button, column, container, row, text, Column, Space};
use iced::{Alignment, Element, Length};

use super::HomeMessage;
use crate::core::models::{format_file_size, format_relative_time, RecordingSummary};
use crate::presentation::design::components::{icon_button, labeled_button, with_tooltip};
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const MAXIMUM_VISIBLE_RECORDINGS: usize = 4;
const RECENT_SECTION_TITLE: &str = "Recent recordings";
const OPEN_FOLDER_LABEL: &str = "Open folder";
const EMPTY_STATE_TEXT: &str = "Recordings you make will show up here";
const PLAY_TOOLTIP: &str = "Play";
const REVEAL_TOOLTIP: &str = "Show in folder";
const META_SEPARATOR: &str = "  ·  ";
const LIST_ITEM_SPACING: f32 = 2.0;

pub(super) fn render_recent_recordings<'a>(
    recordings: &'a [RecordingSummary],
    now: SystemTime,
) -> Element<'a, HomeMessage> {
    let section_header = row![
        text(RECENT_SECTION_TITLE).size(font_size::HEADING),
        Space::new().width(Length::Fill),
        labeled_button(
            Icon::Folder,
            OPEN_FOLDER_LABEL,
            HomeMessage::OpenRecordingsFolder,
            ButtonTone::Ghost,
            IconTint::TextSecondary,
        ),
    ]
    .align_y(Alignment::Center);
    column![section_header, render_recording_list(recordings, now)]
        .spacing(spacing::SMALL)
        .into()
}

fn render_recording_list<'a>(
    recordings: &'a [RecordingSummary],
    now: SystemTime,
) -> Element<'a, HomeMessage> {
    if recordings.is_empty() {
        return render_empty_state();
    }
    let rows: Vec<Element<'a, HomeMessage>> = recordings
        .iter()
        .take(MAXIMUM_VISIBLE_RECORDINGS)
        .map(|recording| render_recording_row(recording, now))
        .collect();
    container(Column::with_children(rows).spacing(LIST_ITEM_SPACING))
        .padding(spacing::EXTRA_SMALL)
        .style(surfaces::card)
        .into()
}

fn render_empty_state<'a>() -> Element<'a, HomeMessage> {
    container(
        row![
            icon(Icon::Film, icon_size::MEDIUM, IconTint::TextMuted),
            text(EMPTY_STATE_TEXT)
                .size(font_size::CAPTION)
                .style(surfaces::muted_text),
        ]
        .spacing(spacing::MEDIUM)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(spacing::LARGE + 2.0)
    .style(surfaces::card)
    .into()
}

fn render_recording_row<'a>(
    recording: &'a RecordingSummary,
    now: SystemTime,
) -> Element<'a, HomeMessage> {
    let meta_text = format!(
        "{}{}{}",
        format_relative_time(recording.modified_at, now),
        META_SEPARATOR,
        format_file_size(recording.size_bytes)
    );
    let details = column![
        text(recording.file_name.as_str()).size(font_size::BODY),
        text(meta_text)
            .size(font_size::SMALL)
            .style(surfaces::muted_text),
    ]
    .spacing(LIST_ITEM_SPACING);
    let row_content = row![
        icon(Icon::Film, icon_size::MEDIUM, IconTint::Accent),
        details,
        Space::new().width(Length::Fill),
        render_recording_actions(recording)
    ]
    .spacing(spacing::MEDIUM)
    .align_y(Alignment::Center);
    button(row_content)
        .width(Length::Fill)
        .padding([spacing::EXTRA_SMALL, spacing::MEDIUM])
        .style(button_style(ButtonTone::ListRow))
        .on_press(HomeMessage::OpenRecording(recording.file_path.clone()))
        .into()
}

fn render_recording_actions<'a>(recording: &'a RecordingSummary) -> Element<'a, HomeMessage> {
    row![
        with_tooltip(
            icon_button(
                Icon::Play,
                HomeMessage::OpenRecording(recording.file_path.clone()),
                ButtonTone::Ghost,
                IconTint::TextSecondary,
            ),
            PLAY_TOOLTIP,
            iced::widget::tooltip::Position::Top,
        ),
        with_tooltip(
            icon_button(
                Icon::Folder,
                HomeMessage::RevealRecording(recording.file_path.clone()),
                ButtonTone::Ghost,
                IconTint::TextSecondary,
            ),
            REVEAL_TOOLTIP,
            iced::widget::tooltip::Position::Top,
        ),
    ]
    .into()
}
