use iced::{Color, Theme};

use crate::core::models::ThemeMode;

pub const DARK_THEME_NAME: &str = "Circle Dark";
pub const LIGHT_THEME_NAME: &str = "Circle Light";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesignTokens {
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_hover: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_soft: Color,
    pub on_accent: Color,
    pub record: Color,
    pub record_hover: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

pub const DARK_TOKENS: DesignTokens = DesignTokens {
    background: Color::from_rgb8(0x0E, 0x0F, 0x13),
    surface: Color::from_rgb8(0x16, 0x17, 0x1D),
    surface_raised: Color::from_rgb8(0x1E, 0x20, 0x27),
    surface_hover: Color::from_rgb8(0x26, 0x28, 0x31),
    border: Color::from_rgb8(0x2A, 0x2D, 0x36),
    border_strong: Color::from_rgb8(0x3A, 0x3E, 0x4A),
    text_primary: Color::from_rgb8(0xF4, 0xF5, 0xF7),
    text_secondary: Color::from_rgb8(0xA7, 0xAB, 0xB8),
    text_muted: Color::from_rgb8(0x6E, 0x73, 0x82),
    accent: Color::from_rgb8(0x7B, 0x4D, 0xFF),
    accent_hover: Color::from_rgb8(0x93, 0x6B, 0xFF),
    accent_pressed: Color::from_rgb8(0x66, 0x38, 0xF0),
    accent_soft: Color::from_rgb8(0x2A, 0x1F, 0x5C),
    on_accent: Color::WHITE,
    record: Color::from_rgb8(0xFF, 0x2E, 0x55),
    record_hover: Color::from_rgb8(0xFF, 0x4D, 0x6D),
    success: Color::from_rgb8(0x1E, 0xE0, 0x7A),
    warning: Color::from_rgb8(0xFF, 0xC4, 0x00),
    danger: Color::from_rgb8(0xFF, 0x5C, 0x5C),
};

pub const LIGHT_TOKENS: DesignTokens = DesignTokens {
    background: Color::from_rgb8(0xF5, 0xF6, 0xF8),
    surface: Color::from_rgb8(0xFF, 0xFF, 0xFF),
    surface_raised: Color::from_rgb8(0xFF, 0xFF, 0xFF),
    surface_hover: Color::from_rgb8(0xEE, 0xEF, 0xF3),
    border: Color::from_rgb8(0xE2, 0xE4, 0xEA),
    border_strong: Color::from_rgb8(0xC9, 0xCD, 0xD6),
    text_primary: Color::from_rgb8(0x14, 0x16, 0x1C),
    text_secondary: Color::from_rgb8(0x55, 0x5B, 0x6A),
    text_muted: Color::from_rgb8(0x8A, 0x8F, 0x9C),
    accent: Color::from_rgb8(0x6A, 0x3D, 0xFF),
    accent_hover: Color::from_rgb8(0x7D, 0x55, 0xFF),
    accent_pressed: Color::from_rgb8(0x58, 0x2C, 0xE6),
    accent_soft: Color::from_rgb8(0xEE, 0xEA, 0xFF),
    on_accent: Color::WHITE,
    record: Color::from_rgb8(0xFF, 0x1F, 0x4B),
    record_hover: Color::from_rgb8(0xFF, 0x40, 0x66),
    success: Color::from_rgb8(0x00, 0xC2, 0x5E),
    warning: Color::from_rgb8(0xD9, 0x8A, 0x00),
    danger: Color::from_rgb8(0xE0, 0x3E, 0x3E),
};

pub const HUD_TOKENS: DesignTokens = DesignTokens {
    background: Color::from_rgba8(0x0E, 0x0F, 0x13, 0.92),
    surface: Color::from_rgba8(0x16, 0x17, 0x1D, 0.94),
    surface_raised: Color::from_rgba8(0x24, 0x26, 0x2E, 0.96),
    surface_hover: Color::from_rgba8(0x30, 0x33, 0x3D, 0.98),
    ..DARK_TOKENS
};

pub mod vibrant {
    use iced::Color;

    pub const VIOLET: Color = Color::from_rgb8(0x7B, 0x4D, 0xFF);
    pub const RED: Color = Color::from_rgb8(0xFF, 0x2E, 0x55);
    pub const BLUE: Color = Color::from_rgb8(0x1F, 0x8B, 0xFF);
    pub const GREEN: Color = Color::from_rgb8(0x1E, 0xE0, 0x7A);
    pub const YELLOW: Color = Color::from_rgb8(0xFF, 0xD6, 0x00);
    pub const ORANGE: Color = Color::from_rgb8(0xFF, 0x7A, 0x00);
    pub const PINK: Color = Color::from_rgb8(0xFF, 0x2D, 0xA6);
    pub const WHITE: Color = Color::WHITE;
}

pub mod spacing {
    pub const EXTRA_SMALL: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    pub const EXTRA_LARGE: f32 = 24.0;
    pub const HUGE: f32 = 32.0;
}

pub mod radius {
    pub const ROOT_FONT_SIZE: f32 = 16.0;
    pub const STANDARD: f32 = 0.175 * ROOT_FONT_SIZE;
    pub const SMALL: f32 = STANDARD;
    pub const MEDIUM: f32 = STANDARD;
    pub const LARGE: f32 = STANDARD;
    pub const PILL: f32 = STANDARD;
}

pub mod font_size {
    pub const DISPLAY: f32 = 28.0;
    pub const TITLE: f32 = 20.0;
    pub const HEADING: f32 = 16.0;
    pub const BODY: f32 = 14.0;
    pub const CAPTION: f32 = 12.5;
    pub const SMALL: f32 = 11.5;
}

pub mod icon_size {
    pub const SMALL: f32 = 14.0;
    pub const MEDIUM: f32 = 18.0;
    pub const LARGE: f32 = 22.0;
    pub const HERO: f32 = 28.0;
}

pub fn tokens_for_mode(mode: &ThemeMode) -> &'static DesignTokens {
    match mode {
        ThemeMode::Dark => &DARK_TOKENS,
        ThemeMode::Light => &LIGHT_TOKENS,
    }
}

pub fn tokens_for_theme(theme: &Theme) -> &'static DesignTokens {
    match theme.to_string().as_str() {
        LIGHT_THEME_NAME | OVERLAY_LIGHT_THEME_NAME => &LIGHT_TOKENS,
        _ => &DARK_TOKENS,
    }
}

pub fn build_theme(mode: &ThemeMode) -> Theme {
    let tokens = tokens_for_mode(mode);
    let theme_name = match mode {
        ThemeMode::Dark => DARK_THEME_NAME,
        ThemeMode::Light => LIGHT_THEME_NAME,
    };
    Theme::custom(
        theme_name.to_string(),
        iced::theme::Palette {
            background: tokens.background,
            text: tokens.text_primary,
            primary: tokens.accent,
            success: tokens.success,
            danger: tokens.danger,
            warning: tokens.warning,
        },
    )
}

pub const OVERLAY_DARK_THEME_NAME: &str = "Circle Overlay Dark";
pub const OVERLAY_LIGHT_THEME_NAME: &str = "Circle Overlay Light";

pub fn build_overlay_theme(mode: &ThemeMode) -> Theme {
    let theme_name = match mode {
        ThemeMode::Dark => OVERLAY_DARK_THEME_NAME,
        ThemeMode::Light => OVERLAY_LIGHT_THEME_NAME,
    };
    Theme::custom(
        theme_name.to_string(),
        iced::theme::Palette {
            background: Color::TRANSPARENT,
            text: HUD_TOKENS.text_primary,
            primary: HUD_TOKENS.accent,
            success: HUD_TOKENS.success,
            danger: HUD_TOKENS.danger,
            warning: HUD_TOKENS.warning,
        },
    )
}

pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

#[cfg(test)]
mod tests;
