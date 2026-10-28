use iced::widget::{container, text};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

use super::tokens::{radius, tokens_for_theme, DesignTokens, HUD_TOKENS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
}

pub fn window_background(theme: &Theme) -> container::Style {
    let tokens = tokens_for_theme(theme);
    container::Style {
        background: Some(Background::Color(tokens.background)),
        text_color: Some(tokens.text_primary),
        ..Default::default()
    }
}

pub fn card(theme: &Theme) -> container::Style {
    let tokens = tokens_for_theme(theme);
    bordered_surface(tokens.surface, tokens.border, radius::LARGE, tokens)
}

pub fn floating_card(theme: &Theme) -> container::Style {
    let tokens = tokens_for_theme(theme);
    bordered_surface(
        tokens.background,
        tokens.border_strong,
        radius::LARGE,
        tokens,
    )
}

pub fn raised_card(theme: &Theme) -> container::Style {
    let tokens = tokens_for_theme(theme);
    bordered_surface(tokens.surface_raised, tokens.border, radius::MEDIUM, tokens)
}

pub fn hud_panel(_theme: &Theme) -> container::Style {
    container::Style {
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..bordered_surface(
            HUD_TOKENS.surface,
            HUD_TOKENS.border_strong,
            radius::LARGE,
            &HUD_TOKENS,
        )
    }
}

pub fn hud_pill(_theme: &Theme) -> container::Style {
    container::Style {
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 14.0,
        },
        ..bordered_surface(
            HUD_TOKENS.surface,
            HUD_TOKENS.border_strong,
            radius::PILL,
            &HUD_TOKENS,
        )
    }
}

pub fn keycap(theme: &Theme) -> container::Style {
    let tokens = tokens_for_theme(theme);
    bordered_surface(
        tokens.surface_raised,
        tokens.border_strong,
        radius::STANDARD,
        tokens,
    )
}

pub fn hud_keycap(_theme: &Theme) -> container::Style {
    bordered_surface(
        HUD_TOKENS.surface_raised,
        HUD_TOKENS.border_strong,
        radius::STANDARD,
        &HUD_TOKENS,
    )
}

pub fn tooltip(_theme: &Theme) -> container::Style {
    bordered_surface(
        HUD_TOKENS.surface_raised,
        HUD_TOKENS.border_strong,
        radius::SMALL,
        &HUD_TOKENS,
    )
}

pub fn toast(kind: ToastKind) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| {
        let accent = match kind {
            ToastKind::Success => HUD_TOKENS.success,
            ToastKind::Error => HUD_TOKENS.danger,
        };
        container::Style {
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..bordered_surface(HUD_TOKENS.surface, accent, radius::PILL, &HUD_TOKENS)
        }
    }
}

pub fn secondary_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens_for_theme(theme).text_secondary),
    }
}

pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens_for_theme(theme).text_muted),
    }
}

pub fn accent_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens_for_theme(theme).accent_hover),
    }
}

pub fn hud_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(HUD_TOKENS.text_primary),
    }
}

pub fn hud_secondary_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(HUD_TOKENS.text_secondary),
    }
}

pub fn colored_text(color: Color) -> impl Fn(&Theme) -> text::Style {
    move |_theme: &Theme| text::Style { color: Some(color) }
}

fn bordered_surface(
    background: Color,
    border_color: Color,
    corner_radius: f32,
    tokens: &DesignTokens,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(tokens.text_primary),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: corner_radius.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
