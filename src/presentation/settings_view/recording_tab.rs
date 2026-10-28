use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Length};

use super::controls::{segmented, setting_row, settings_card, switch_row};
use super::{SettingsChange, SettingsMessage};
use crate::core::models::{
    CameraBubbleShape, CameraBubbleSize, CountdownChoice, FrameRateChoice, RecordingPreferences,
    RecordingQuality, UserSettings,
};
use crate::presentation::design::{button_style, font_size, spacing, surfaces, ButtonTone};

mod labels;

use labels::*;

pub fn render_recording_tab<'a>(draft: &'a UserSettings) -> Vec<Element<'a, SettingsMessage>> {
    let preferences = draft.recording_preferences;
    vec![
        render_video_card(preferences),
        render_effects_card(preferences),
        render_camera_card(preferences),
        render_audio_card(preferences),
        render_saving_card(draft),
    ]
}

fn render_video_card<'a>(preferences: RecordingPreferences) -> Element<'a, SettingsMessage> {
    settings_card(
        VIDEO_TITLE,
        vec![
            setting_row(
                FRAME_RATE_LABEL,
                FRAME_RATE_DESCRIPTION,
                segmented(
                    &[
                        (FrameRateChoice::Thirty, THIRTY_FPS_LABEL),
                        (FrameRateChoice::Sixty, SIXTY_FPS_LABEL),
                    ],
                    preferences.frame_rate,
                    |choice| SettingsMessage::Change(SettingsChange::FrameRate(choice)),
                ),
            ),
            setting_row(
                QUALITY_LABEL,
                QUALITY_DESCRIPTION,
                segmented(
                    &[
                        (RecordingQuality::Standard, STANDARD_LABEL),
                        (RecordingQuality::Crisp, CRISP_LABEL),
                    ],
                    preferences.quality,
                    |choice| SettingsMessage::Change(SettingsChange::Quality(choice)),
                ),
            ),
            render_countdown_row(preferences),
        ],
    )
}

fn render_countdown_row<'a>(preferences: RecordingPreferences) -> Element<'a, SettingsMessage> {
    setting_row(
        COUNTDOWN_LABEL,
        COUNTDOWN_DESCRIPTION,
        segmented(
            &[
                (CountdownChoice::Off, OFF_LABEL),
                (CountdownChoice::Three, THREE_SECONDS_LABEL),
                (CountdownChoice::Five, FIVE_SECONDS_LABEL),
            ],
            preferences.countdown,
            |choice| SettingsMessage::Change(SettingsChange::Countdown(choice)),
        ),
    )
}

fn render_effects_card<'a>(preferences: RecordingPreferences) -> Element<'a, SettingsMessage> {
    settings_card(
        EFFECTS_TITLE,
        vec![
            switch_row(
                CLICKS_LABEL,
                CLICKS_DESCRIPTION,
                preferences.click_highlights_enabled,
                |enabled| SettingsMessage::Change(SettingsChange::ClickHighlights(enabled)),
            ),
            switch_row(
                SPOTLIGHT_LABEL,
                SPOTLIGHT_DESCRIPTION,
                preferences.cursor_spotlight_enabled,
                |enabled| SettingsMessage::Change(SettingsChange::CursorSpotlight(enabled)),
            ),
        ],
    )
}

fn render_camera_card<'a>(preferences: RecordingPreferences) -> Element<'a, SettingsMessage> {
    settings_card(
        CAMERA_TITLE,
        vec![
            setting_row(
                BUBBLE_SIZE_LABEL,
                BUBBLE_SIZE_DESCRIPTION,
                segmented(
                    &[
                        (CameraBubbleSize::Small, SMALL_LABEL),
                        (CameraBubbleSize::Medium, MEDIUM_LABEL),
                        (CameraBubbleSize::Large, LARGE_LABEL),
                    ],
                    preferences.camera_bubble_size,
                    |choice| SettingsMessage::Change(SettingsChange::CameraBubbleSize(choice)),
                ),
            ),
            setting_row(
                BUBBLE_SHAPE_LABEL,
                BUBBLE_SHAPE_DESCRIPTION,
                segmented(
                    &[
                        (CameraBubbleShape::Circle, CIRCLE_LABEL),
                        (CameraBubbleShape::RoundedSquare, ROUNDED_LABEL),
                    ],
                    preferences.camera_bubble_shape,
                    |choice| SettingsMessage::Change(SettingsChange::CameraBubbleShape(choice)),
                ),
            ),
        ],
    )
}

fn render_audio_card<'a>(preferences: RecordingPreferences) -> Element<'a, SettingsMessage> {
    settings_card(
        AUDIO_TITLE,
        vec![
            switch_row(
                MICROPHONE_LABEL,
                MICROPHONE_DESCRIPTION,
                preferences.microphone_enabled,
                |enabled| SettingsMessage::Change(SettingsChange::MicrophoneByDefault(enabled)),
            ),
            switch_row(
                SYSTEM_AUDIO_LABEL,
                SYSTEM_AUDIO_DESCRIPTION,
                preferences.system_audio_enabled,
                |enabled| SettingsMessage::Change(SettingsChange::SystemAudioByDefault(enabled)),
            ),
        ],
    )
}

fn render_saving_card(draft: &UserSettings) -> Element<'_, SettingsMessage> {
    let folder_row = row![
        column![
            text(FOLDER_LABEL).size(font_size::BODY),
            text(draft.recording_save_location.as_str())
                .size(font_size::CAPTION)
                .style(surfaces::muted_text),
        ]
        .spacing(FOLDER_TEXT_SPACING)
        .width(Length::Fill),
        button(text(CHOOSE_FOLDER_LABEL).size(font_size::CAPTION))
            .padding([spacing::EXTRA_SMALL + 2.0, spacing::MEDIUM])
            .style(button_style(ButtonTone::Secondary))
            .on_press(SettingsMessage::ChooseRecordingsFolder),
    ]
    .spacing(spacing::LARGE)
    .align_y(Alignment::Center);
    settings_card(SAVE_TITLE, vec![folder_row.into()])
}
