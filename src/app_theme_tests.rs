#[cfg(test)]
mod tests {
    use crate::app_theme::*;
    use crate::user_settings::ThemeMode;
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
    fn test_button_hover_increases_shadow_blur() {
        let theme = Theme::Dark;

        let active_style = primary_button_style(&theme, button::Status::Active);
        let hover_style = primary_button_style(&theme, button::Status::Hovered);

        assert!(hover_style.shadow.blur_radius > active_style.shadow.blur_radius);
    }
}
