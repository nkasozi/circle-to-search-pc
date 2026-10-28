use iced::widget::{column, container, row, text, toggler, Column};
use iced::{Alignment, Element, Length};

use super::{OnboardingMessage, OnboardingView};
use crate::presentation::design::components::labeled_button;
use crate::presentation::design::inputs::switch_style;
use crate::presentation::design::tokens::vibrant;
use crate::presentation::design::{
    font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

mod constants;
mod widgets;

use constants::*;
use widgets::{feature_row, heading, hero_badge, shortcut_row};

impl OnboardingView {
    pub(super) fn render_welcome_step(&self) -> Element<'_, OnboardingMessage> {
        column![
            hero_badge(Icon::ScanText, vibrant::VIOLET),
            heading(WELCOME_TITLE, WELCOME_SUBTITLE),
            column![
                feature_row(
                    Icon::Search,
                    vibrant::VIOLET,
                    FEATURE_SEARCH_TITLE,
                    FEATURE_SEARCH_TEXT
                ),
                feature_row(
                    Icon::Record,
                    vibrant::RED,
                    FEATURE_RECORD_TITLE,
                    FEATURE_RECORD_TEXT
                ),
                feature_row(
                    Icon::Pen,
                    vibrant::BLUE,
                    FEATURE_DRAW_TITLE,
                    FEATURE_DRAW_TEXT
                ),
            ]
            .spacing(spacing::LARGE),
        ]
        .spacing(spacing::EXTRA_LARGE)
        .into()
    }

    pub(super) fn render_screen_access_step(&self) -> Element<'_, OnboardingMessage> {
        let status: Element<'_, OnboardingMessage> = match self.screen_access_granted {
            true => render_screen_access_granted_status(),
            false => self.render_screen_access_actions(),
        };
        column![
            hero_badge(Icon::Shield, vibrant::GREEN),
            heading(SCREEN_TITLE, SCREEN_TEXT),
            container(status)
                .padding(spacing::LARGE)
                .width(Length::Fill)
                .style(surfaces::card),
            text(DEVICES_NOTE)
                .size(font_size::CAPTION)
                .style(surfaces::muted_text),
        ]
        .spacing(spacing::EXTRA_LARGE)
        .into()
    }

    fn render_screen_access_actions(&self) -> Element<'_, OnboardingMessage> {
        let actions = column![render_screen_access_buttons()].spacing(spacing::MEDIUM);
        match self.screen_access_requested {
            true => append_screen_access_waiting_hints(actions).into(),
            false => actions.into(),
        }
    }

    pub(super) fn render_shortcuts_step(&self) -> Element<'_, OnboardingMessage> {
        column![
            hero_badge(Icon::Keyboard, vibrant::BLUE),
            heading(SHORTCUTS_TITLE, SHORTCUTS_TEXT),
            shortcut_row(
                SEARCH_SHORTCUT_LABEL,
                &self.capture_hotkey,
                Icon::ScanText,
                vibrant::VIOLET
            ),
            shortcut_row(
                RECORD_SHORTCUT_LABEL,
                &self.recording_hotkey,
                Icon::Record,
                vibrant::RED
            ),
        ]
        .spacing(spacing::LARGE)
        .into()
    }

    pub(super) fn render_startup_step(&self) -> Element<'_, OnboardingMessage> {
        let switches = column![
            toggler(self.launch_at_login)
                .label(LAUNCH_AT_LOGIN_LABEL)
                .text_size(font_size::BODY)
                .style(switch_style)
                .on_toggle(OnboardingMessage::ToggleLaunchAtLogin),
            toggler(self.show_home_on_launch)
                .label(SHOW_HOME_LABEL)
                .text_size(font_size::BODY)
                .style(switch_style)
                .on_toggle(OnboardingMessage::ToggleShowHomeOnLaunch),
        ]
        .spacing(spacing::LARGE);
        column![
            hero_badge(Icon::Power, vibrant::ORANGE),
            heading(STARTUP_TITLE, STARTUP_TEXT),
            container(switches)
                .padding(spacing::LARGE)
                .width(Length::Fill)
                .style(surfaces::card),
        ]
        .spacing(spacing::EXTRA_LARGE)
        .into()
    }
}

fn render_screen_access_granted_status<'a>() -> Element<'a, OnboardingMessage> {
    row![
        icon(Icon::Check, icon_size::MEDIUM, IconTint::Success),
        text(SCREEN_GRANTED_TEXT).size(font_size::BODY)
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center)
    .into()
}

fn render_screen_access_buttons<'a>() -> Element<'a, OnboardingMessage> {
    row![
        labeled_button(
            Icon::Shield,
            SCREEN_ALLOW_LABEL,
            OnboardingMessage::RequestScreenAccess,
            ButtonTone::Primary,
            iced::Color::WHITE
        ),
        labeled_button(
            Icon::ExternalLink,
            SCREEN_OPEN_SETTINGS_LABEL,
            OnboardingMessage::OpenScreenRecordingSettings,
            ButtonTone::Secondary,
            IconTint::TextPrimary
        ),
    ]
    .spacing(spacing::SMALL)
    .into()
}

fn append_screen_access_waiting_hints(
    actions: Column<'_, OnboardingMessage>,
) -> Column<'_, OnboardingMessage> {
    let restart_row = row![
        text(SCREEN_RESTART_HINT)
            .size(font_size::CAPTION)
            .style(surfaces::secondary_text)
            .width(Length::Fill),
        labeled_button(
            Icon::Refresh,
            SCREEN_RESTART_LABEL,
            OnboardingMessage::RestartApp,
            ButtonTone::Ghost,
            IconTint::TextSecondary
        ),
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    actions
        .push(
            text(SCREEN_WAITING_TEXT)
                .size(font_size::CAPTION)
                .style(surfaces::accent_text),
        )
        .push(restart_row)
}
