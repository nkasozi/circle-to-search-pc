use super::*;

#[test]
fn test_tokens_for_theme_resolves_light_theme_by_name() {
    let theme = build_theme(&ThemeMode::Light);
    assert_eq!(tokens_for_theme(&theme), &LIGHT_TOKENS);
}

#[test]
fn test_tokens_for_theme_defaults_to_dark() {
    let theme = build_theme(&ThemeMode::Dark);
    assert_eq!(tokens_for_theme(&theme), &DARK_TOKENS);
    assert_eq!(tokens_for_theme(&Theme::Nord), &DARK_TOKENS);
}

#[test]
fn test_with_alpha_keeps_rgb() {
    let faded = with_alpha(DARK_TOKENS.accent, 0.25);
    assert_eq!(faded.r, DARK_TOKENS.accent.r);
    assert_eq!(faded.a, 0.25);
}
