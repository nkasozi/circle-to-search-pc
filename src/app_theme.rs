use iced::{Background, Border, Color, Shadow, Theme, Vector};
use iced::widget::button;

use crate::user_settings::ThemeMode;

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
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.122, 0.655, 0.408))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.122, 0.655, 0.408),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.098, 0.529, 0.329, 0.4),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 12.0,
            },
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.078, 0.420, 0.263))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.078, 0.420, 0.263),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            text_color: Color::from_rgb(0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
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
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.541, 0.341, 0.847))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.541, 0.341, 0.847),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.435, 0.259, 0.757, 0.4),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 12.0,
            },
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.357, 0.208, 0.627))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.357, 0.208, 0.627),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            text_color: Color::from_rgb(0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.9, 0.3, 0.3))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(1.0, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(1.0, 0.4, 0.4))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(1.0, 0.5, 0.5),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.9, 0.3, 0.3, 0.4),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 12.0,
            },
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(0.9, 0.3, 0.3),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            text_color: Color::from_rgb(0.5, 0.5, 0.5),
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 12.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}
