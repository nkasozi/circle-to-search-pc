use super::*;
use crate::core::models::ThemeMode;
use crate::presentation::design::tokens::{build_theme, DARK_TOKENS, LIGHT_TOKENS};

#[test]
fn test_primary_button_uses_accent_background() {
    let style =
        button_style(ButtonTone::Primary)(&build_theme(&ThemeMode::Dark), button::Status::Active);
    assert_eq!(
        style.background,
        Some(Background::Color(DARK_TOKENS.accent))
    );
}

#[test]
fn test_primary_button_follows_light_theme() {
    let style =
        button_style(ButtonTone::Primary)(&build_theme(&ThemeMode::Light), button::Status::Hovered);
    assert_eq!(
        style.background,
        Some(Background::Color(LIGHT_TOKENS.accent_hover))
    );
}

#[test]
fn test_disabled_button_uses_muted_text() {
    let style = button_style(ButtonTone::Secondary)(
        &build_theme(&ThemeMode::Dark),
        button::Status::Disabled,
    );
    assert_eq!(style.text_color, DARK_TOKENS.text_muted);
}

#[test]
fn test_hud_buttons_ignore_light_theme() {
    let style =
        button_style(ButtonTone::HudIcon)(&build_theme(&ThemeMode::Light), button::Status::Active);
    assert_eq!(style.text_color, HUD_TOKENS.text_primary);
}
