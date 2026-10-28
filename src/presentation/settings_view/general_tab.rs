use iced::widget::{button, row, text};
use iced::{Alignment, Element};

use super::controls::{segmented, setting_row, settings_card, switch_row};
use super::{SettingsChange, SettingsMessage, SettingsViewState, ShortcutCaptureOutcome};
use crate::core::models::{GlobalHotkeyAction, ThemeMode, UserSettings};
use crate::presentation::design::components::{shortcut_keycaps, KeycapSurface};
use crate::presentation::design::{button_style, font_size, spacing, surfaces, ButtonTone};

const SHORTCUTS_TITLE: &str = "Shortcuts";
const SEARCH_SHORTCUT_LABEL: &str = "Search your screen";
const SEARCH_SHORTCUT_DESCRIPTION: &str = "Opens the circle-to-search overlay";
const RECORD_SHORTCUT_LABEL: &str = "Start or stop recording";
const RECORD_SHORTCUT_DESCRIPTION: &str = "Opens the recorder, or stops a recording";
const CHANGE_LABEL: &str = "Change";
const CANCEL_LABEL: &str = "Cancel";
const PRESS_SHORTCUT_LABEL: &str = "Press a shortcut…";
const NEEDS_MODIFIER_LABEL: &str = "Add ⌘, ⌃ or ⌥";
const UNSUPPORTED_KEY_LABEL: &str = "Try a letter or number";
const APPEARANCE_TITLE: &str = "Appearance";
const THEME_LABEL: &str = "Theme";
const THEME_DESCRIPTION: &str = "Applies to every window";
const DARK_LABEL: &str = "Dark";
const LIGHT_LABEL: &str = "Light";
const STARTUP_TITLE: &str = "Startup";
const LAUNCH_AT_LOGIN_LABEL: &str = "Open at login";
const LAUNCH_AT_LOGIN_DESCRIPTION: &str = "Be ready the moment you need it";
const SHOW_HOME_LABEL: &str = "Show home window on start";
const SHOW_HOME_DESCRIPTION: &str = "Otherwise it waits quietly in the menu bar";

fn shortcut_control<'a>(
    state: &SettingsViewState,
    action: GlobalHotkeyAction,
    accelerator: &'a str,
) -> Element<'a, SettingsMessage> {
    if state.capturing_shortcut == Some(action) {
        let prompt = match state.shortcut_hint {
            Some(ShortcutCaptureOutcome::NeedsModifier) => NEEDS_MODIFIER_LABEL,
            Some(ShortcutCaptureOutcome::Unsupported) => UNSUPPORTED_KEY_LABEL,
            _ => PRESS_SHORTCUT_LABEL,
        };
        return row![
            text(prompt)
                .size(font_size::CAPTION)
                .style(surfaces::accent_text),
            button(text(CANCEL_LABEL).size(font_size::CAPTION))
                .padding([spacing::EXTRA_SMALL + 2.0, spacing::MEDIUM])
                .style(button_style(ButtonTone::Ghost))
                .on_press(SettingsMessage::CancelShortcutCapture),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center)
        .into();
    }
    row![
        shortcut_keycaps(accelerator, KeycapSurface::Window),
        button(text(CHANGE_LABEL).size(font_size::CAPTION))
            .padding([spacing::EXTRA_SMALL + 2.0, spacing::MEDIUM])
            .style(button_style(ButtonTone::Secondary))
            .on_press(SettingsMessage::StartShortcutCapture(action)),
    ]
    .spacing(spacing::MEDIUM)
    .align_y(Alignment::Center)
    .into()
}

pub fn render_general_tab<'a>(
    state: &'a SettingsViewState,
    draft: &'a UserSettings,
) -> Vec<Element<'a, SettingsMessage>> {
    vec![
        render_shortcuts_card(state, draft),
        render_appearance_card(draft),
        render_startup_card(draft),
    ]
}

fn render_shortcuts_card<'a>(
    state: &'a SettingsViewState,
    draft: &'a UserSettings,
) -> Element<'a, SettingsMessage> {
    settings_card(
        SHORTCUTS_TITLE,
        vec![
            setting_row(
                SEARCH_SHORTCUT_LABEL,
                SEARCH_SHORTCUT_DESCRIPTION,
                shortcut_control(state, GlobalHotkeyAction::Capture, &draft.capture_hotkey),
            ),
            setting_row(
                RECORD_SHORTCUT_LABEL,
                RECORD_SHORTCUT_DESCRIPTION,
                shortcut_control(
                    state,
                    GlobalHotkeyAction::ToggleRecording,
                    &draft.recording_hotkey,
                ),
            ),
        ],
    )
}

fn render_appearance_card<'a>(draft: &'a UserSettings) -> Element<'a, SettingsMessage> {
    settings_card(
        APPEARANCE_TITLE,
        vec![setting_row(
            THEME_LABEL,
            THEME_DESCRIPTION,
            segmented(
                &[
                    (ThemeMode::Dark, DARK_LABEL),
                    (ThemeMode::Light, LIGHT_LABEL),
                ],
                draft.theme_mode,
                |mode| SettingsMessage::Change(SettingsChange::Theme(mode)),
            ),
        )],
    )
}

fn render_startup_card<'a>(draft: &'a UserSettings) -> Element<'a, SettingsMessage> {
    settings_card(
        STARTUP_TITLE,
        vec![
            switch_row(
                LAUNCH_AT_LOGIN_LABEL,
                LAUNCH_AT_LOGIN_DESCRIPTION,
                draft.launch_at_login,
                |enabled| SettingsMessage::Change(SettingsChange::LaunchAtLogin(enabled)),
            ),
            switch_row(
                SHOW_HOME_LABEL,
                SHOW_HOME_DESCRIPTION,
                draft.show_home_on_launch,
                |enabled| SettingsMessage::Change(SettingsChange::ShowHomeOnLaunch(enabled)),
            ),
        ],
    )
}
