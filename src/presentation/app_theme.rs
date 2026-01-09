use iced::widget::button;
use iced::{Background, Border, Color, Shadow, Theme};

use crate::core::models::ThemeMode;

pub fn get_theme(mode: &ThemeMode) -> Theme {
    match mode {
        ThemeMode::Dark => Theme::custom(
            "Dark".to_string(),
            iced::theme::Palette {
                background: Color::from_rgb(0.0, 0.0, 0.0),
                text: Color::from_rgb(1.0, 1.0, 1.0),
                primary: Color::from_rgb(0.4, 0.6, 1.0),
                success: Color::from_rgb(0.2, 0.9, 0.4),
                danger: Color::from_rgb(1.0, 0.3, 0.3),
                warning: Color::from_rgb(1.0, 0.7, 0.0),
            },
        ),
        ThemeMode::Light => Theme::custom(
            "Light".to_string(),
            iced::theme::Palette {
                background: Color::from_rgb(0.95, 0.95, 0.97),
                text: Color::from_rgb(0.1, 0.1, 0.1),
                primary: Color::from_rgb(0.2, 0.4, 0.9),
                success: Color::from_rgb(0.1, 0.7, 0.3),
                danger: Color::from_rgb(0.9, 0.2, 0.2),
                warning: Color::from_rgb(0.9, 0.6, 0.0),
            },
        ),
    }
}

pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let _palette = _theme.palette();

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.098, 0.529, 0.329))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.098, 0.529, 0.329),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.122, 0.655, 0.408))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.122, 0.655, 0.408),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.078, 0.420, 0.263))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.078, 0.420, 0.263),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            text_color: Color::from_rgb(0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn purple_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.435, 0.259, 0.757))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.435, 0.259, 0.757),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.541, 0.341, 0.847))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.541, 0.341, 0.847),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.357, 0.208, 0.627))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.357, 0.208, 0.627),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            text_color: Color::from_rgb(0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.25))),
            text_color: Color::from_rgb(0.7, 0.7, 0.7),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.35, 0.35, 0.35))),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            text_color: Color::from_rgb(0.8, 0.8, 0.8),
            border: Border {
                color: Color::from_rgb(0.35, 0.35, 0.35),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

#[allow(dead_code)]
pub fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.25, 0.25))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.8, 0.25, 0.25),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.9, 0.35, 0.35))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.9, 0.35, 0.35),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.7, 0.2, 0.2))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.7, 0.2, 0.2),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            text_color: Color::from_rgb(0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ThemeMode;
    use iced::widget::button;
    use iced::{Background, Color, Theme};

    #[test]
    fn test_get_theme_dark_mode() {
        let theme = get_theme(&ThemeMode::Dark);
        let palette = theme.palette();

        assert_eq!(palette.background, Color::from_rgb(0.0, 0.0, 0.0));
        assert_eq!(palette.text, Color::from_rgb(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_get_theme_light_mode() {
        let theme = get_theme(&ThemeMode::Light);
        let palette = theme.palette();

        assert_eq!(palette.background, Color::from_rgb(0.95, 0.95, 0.97));
        assert_eq!(palette.text, Color::from_rgb(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_primary_button_style_active_has_green_background() {
        let theme = Theme::Dark;
        let style = primary_button_style(&theme, button::Status::Active);

        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, Color::from_rgb(0.098, 0.529, 0.329));
        } else {
            panic!("Expected background color");
        }

        assert_eq!(style.text_color, Color::WHITE);
    }

    #[test]
    fn test_primary_button_style_hovered_is_lighter_green() {
        let theme = Theme::Dark;
        let style = primary_button_style(&theme, button::Status::Hovered);

        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, Color::from_rgb(0.122, 0.655, 0.408));
        } else {
            panic!("Expected background color");
        }
    }

    #[test]
    fn test_primary_button_style_pressed_is_darker_green() {
        let theme = Theme::Dark;
        let style = primary_button_style(&theme, button::Status::Pressed);

        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, Color::from_rgb(0.078, 0.420, 0.263));
        } else {
            panic!("Expected background color");
        }

        assert!(style.snap);
    }

    #[test]
    fn test_primary_button_style_disabled_is_gray() {
        let theme = Theme::Dark;
        let style = primary_button_style(&theme, button::Status::Disabled);

        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, Color::from_rgb(0.3, 0.3, 0.3));
        } else {
            panic!("Expected background color");
        }

        assert_eq!(style.text_color, Color::from_rgb(0.5, 0.5, 0.5));
    }

    #[test]
    fn test_purple_button_style_active_has_purple_background() {
        let theme = Theme::Dark;
        let style = purple_button_style(&theme, button::Status::Active);

        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, Color::from_rgb(0.435, 0.259, 0.757));
        } else {
            panic!("Expected background color");
        }

        assert_eq!(style.text_color, Color::WHITE);
    }

    #[test]
    fn test_button_styles_have_consistent_border_radius() {
        let theme = Theme::Dark;

        let primary_active = primary_button_style(&theme, button::Status::Active);
        let purple_active = purple_button_style(&theme, button::Status::Active);

        assert_eq!(primary_active.border.radius, 6.0.into());
        assert_eq!(purple_active.border.radius, 6.0.into());
    }

    #[test]
    fn test_button_styles_use_no_shadows() {
        let theme = Theme::Dark;

        let active_style = primary_button_style(&theme, button::Status::Active);
        let hover_style = primary_button_style(&theme, button::Status::Hovered);

        assert_eq!(active_style.shadow.blur_radius, 0.0);
        assert_eq!(hover_style.shadow.blur_radius, 0.0);
    }
}
