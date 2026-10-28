use iced::widget::button;
use iced::{Background, Border, Color, Shadow, Theme};

use super::tokens::{radius, tokens_for_theme, with_alpha, DesignTokens, HUD_TOKENS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Primary,
    Secondary,
    Ghost,
    Danger,
    HudIcon,
    HudIconActive,
    HudRecord,
    Segment,
    SegmentActive,
    Card,
    ListRow,
}

struct ToneColors {
    background: Color,
    hovered_background: Color,
    pressed_background: Color,
    text: Color,
    border: Color,
}

pub fn button_style(tone: ButtonTone) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let tokens = resolve_tokens(theme, tone);
        let colors = tone_colors(tokens, tone);
        let corner_radius = tone_radius(tone);
        compose_style(&colors, status, corner_radius, tokens)
    }
}

fn resolve_tokens(theme: &Theme, tone: ButtonTone) -> &'static DesignTokens {
    match tone {
        ButtonTone::HudIcon | ButtonTone::HudIconActive | ButtonTone::HudRecord => &HUD_TOKENS,
        _ => tokens_for_theme(theme),
    }
}

fn tone_radius(tone: ButtonTone) -> f32 {
    match tone {
        ButtonTone::HudIcon | ButtonTone::HudIconActive | ButtonTone::HudRecord => radius::PILL,
        ButtonTone::Card => radius::LARGE,
        _ => radius::MEDIUM,
    }
}

const DANGER_BACKGROUND_ALPHA: f32 = 0.12;
const DANGER_HOVERED_ALPHA: f32 = 0.2;
const DANGER_PRESSED_ALPHA: f32 = 0.28;
const DANGER_BORDER_ALPHA: f32 = 0.35;

fn tone_colors(tokens: &DesignTokens, tone: ButtonTone) -> ToneColors {
    match tone {
        ButtonTone::Primary => primary_colors(tokens),
        ButtonTone::Secondary | ButtonTone::Segment => secondary_colors(tokens),
        ButtonTone::Card => card_colors(tokens),
        ButtonTone::Ghost | ButtonTone::HudIcon | ButtonTone::ListRow => ghost_colors(tokens),
        ButtonTone::SegmentActive | ButtonTone::HudIconActive => active_colors(tokens),
        ButtonTone::Danger => danger_colors(tokens),
        ButtonTone::HudRecord => record_colors(tokens),
    }
}

fn primary_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: tokens.accent,
        hovered_background: tokens.accent_hover,
        pressed_background: tokens.accent_pressed,
        text: tokens.on_accent,
        border: Color::TRANSPARENT,
    }
}

fn secondary_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: tokens.surface_raised,
        hovered_background: tokens.surface_hover,
        pressed_background: tokens.border,
        text: tokens.text_primary,
        border: tokens.border,
    }
}

fn card_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: tokens.surface,
        hovered_background: tokens.surface_hover,
        pressed_background: tokens.surface_raised,
        text: tokens.text_primary,
        border: tokens.border,
    }
}

fn ghost_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: Color::TRANSPARENT,
        hovered_background: tokens.surface_hover,
        pressed_background: tokens.border,
        text: tokens.text_primary,
        border: Color::TRANSPARENT,
    }
}

fn active_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: tokens.accent_soft,
        hovered_background: tokens.accent_soft,
        pressed_background: tokens.accent_soft,
        text: tokens.accent_hover,
        border: tokens.accent,
    }
}

fn danger_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: with_alpha(tokens.danger, DANGER_BACKGROUND_ALPHA),
        hovered_background: with_alpha(tokens.danger, DANGER_HOVERED_ALPHA),
        pressed_background: with_alpha(tokens.danger, DANGER_PRESSED_ALPHA),
        text: tokens.danger,
        border: with_alpha(tokens.danger, DANGER_BORDER_ALPHA),
    }
}

fn record_colors(tokens: &DesignTokens) -> ToneColors {
    ToneColors {
        background: tokens.record,
        hovered_background: tokens.record_hover,
        pressed_background: tokens.record,
        text: Color::WHITE,
        border: Color::TRANSPARENT,
    }
}

fn compose_style(
    colors: &ToneColors,
    status: button::Status,
    corner_radius: f32,
    tokens: &DesignTokens,
) -> button::Style {
    let background = match status {
        button::Status::Active => colors.background,
        button::Status::Hovered => colors.hovered_background,
        button::Status::Pressed => colors.pressed_background,
        button::Status::Disabled => with_alpha(colors.background, 0.4),
    };
    let text_color = match status {
        button::Status::Disabled => tokens.text_muted,
        _ => colors.text,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: corner_radius.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests;
