use iced::widget::{text_input, toggler};
use iced::{Background, Border, Color, Theme};

use super::tokens::{radius, tokens_for_theme, with_alpha};

const DISABLED_TRACK_ALPHA: f32 = 0.45;
const ENABLED_TRACK_ALPHA: f32 = 1.0;

pub fn switch_style(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let tokens = tokens_for_theme(theme);
    let (is_toggled, is_hovered, is_disabled) = match status {
        toggler::Status::Active { is_toggled } => (is_toggled, false, false),
        toggler::Status::Hovered { is_toggled } => (is_toggled, true, false),
        toggler::Status::Disabled { is_toggled } => (is_toggled, false, true),
    };
    let track_color = match (is_toggled, is_hovered) {
        (true, true) => tokens.accent_hover,
        (true, false) => tokens.accent,
        (false, true) => tokens.border_strong,
        (false, false) => tokens.border,
    };
    let track_alpha = match is_disabled {
        true => DISABLED_TRACK_ALPHA,
        false => ENABLED_TRACK_ALPHA,
    };
    toggler::Style {
        background: Background::Color(with_alpha(track_color, track_alpha)),
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        foreground: Background::Color(Color::WHITE),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: Some(tokens.text_primary),
        border_radius: Some(radius::STANDARD.into()),
        padding_ratio: 0.12,
    }
}

pub fn field_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let tokens = tokens_for_theme(theme);
    let border_color = match status {
        text_input::Status::Focused { .. } => tokens.accent,
        text_input::Status::Hovered => tokens.border_strong,
        text_input::Status::Active | text_input::Status::Disabled => tokens.border,
    };
    text_input::Style {
        background: Background::Color(tokens.surface_raised),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::SMALL.into(),
        },
        icon: tokens.text_muted,
        placeholder: tokens.text_muted,
        value: tokens.text_primary,
        selection: with_alpha(tokens.accent, 0.35),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ThemeMode;
    use crate::presentation::design::tokens::{build_theme, DARK_TOKENS};

    #[test]
    fn test_switch_on_uses_accent_track() {
        let style = switch_style(
            &build_theme(&ThemeMode::Dark),
            toggler::Status::Active { is_toggled: true },
        );
        assert_eq!(style.background, Background::Color(DARK_TOKENS.accent));
    }

    #[test]
    fn test_focused_field_uses_accent_border() {
        let style = field_style(
            &build_theme(&ThemeMode::Dark),
            text_input::Status::Focused { is_hovered: false },
        );
        assert_eq!(style.border.color, DARK_TOKENS.accent);
    }
}

pub fn hud_slider_style(
    _theme: &Theme,
    status: iced::widget::slider::Status,
) -> iced::widget::slider::Style {
    use super::tokens::HUD_TOKENS;
    use iced::widget::slider::{Handle, HandleShape, Rail, Style};
    let handle_color = match status {
        iced::widget::slider::Status::Dragged | iced::widget::slider::Status::Hovered => {
            HUD_TOKENS.accent_hover
        }
        iced::widget::slider::Status::Active => Color::WHITE,
    };
    Style {
        rail: Rail {
            backgrounds: (
                Background::Color(HUD_TOKENS.accent),
                Background::Color(HUD_TOKENS.border_strong),
            ),
            width: 4.0,
            border: Border {
                radius: radius::STANDARD.into(),
                ..Border::default()
            },
        },
        handle: Handle {
            shape: HandleShape::Rectangle {
                width: 10,
                border_radius: radius::STANDARD.into(),
            },
            background: Background::Color(handle_color),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}
