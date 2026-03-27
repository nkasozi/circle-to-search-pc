use super::*;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Color, Element, Length};

impl AppOrchestrator {
    pub fn render_main_window(&self) -> Element<'_, OrchestratorMessage> {
        let theme = app_theme::get_theme(&self.settings.theme_mode);

        let logo_icon = text(global_constants::MAIN_WINDOW_ICON_SEARCH).size(64);
        let title = text(global_constants::APPLICATION_TITLE).size(36);
        let subtitle =
            text(global_constants::MAIN_WINDOW_SUBTITLE)
                .size(16)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                });
        let header_section = column![logo_icon, title, subtitle]
            .spacing(8)
            .align_x(Alignment::Center);

        let capture_btn = button(
            row![
                text(global_constants::MAIN_WINDOW_ICON_CAPTURE).size(24),
                text(global_constants::MAIN_WINDOW_CAPTURE_BUTTON_LABEL).size(18)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding([16, 48])
        .style(|theme, status| app_theme::primary_button_style(theme, status))
        .on_press(OrchestratorMessage::CaptureScreen);

        let hotkey_text = container(
            text(global_constants::MAIN_WINDOW_HOTKEY_HINT_TEMPLATE.replace(
                global_constants::MAIN_WINDOW_HOTKEY_TEMPLATE_TOKEN,
                &self.settings.capture_hotkey,
            ))
            .size(13)
            .center()
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
            }),
        )
        .width(Length::Fill)
        .center_x(Length::Fill);

        let or_text = container(
            text(global_constants::MAIN_WINDOW_OR_TEXT)
                .size(13)
                .center()
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
                }),
        )
        .width(Length::Fill)
        .center_x(Length::Fill);

        let action_content = column![hotkey_text, or_text, capture_btn]
            .spacing(12)
            .align_x(Alignment::Center)
            .width(Length::Fill);

        let action_panel = container(action_content)
            .padding([28, 32])
            .width(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Shrink)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: iced::Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                text_color: None,
                snap: false,
            });

        let status_indicator = self.render_status_indicator();

        let system_tray_row = row![
            iced::widget::checkbox(self.settings.run_in_system_tray)
                .on_toggle(OrchestratorMessage::UpdateSystemTrayMode),
            text(global_constants::MAIN_WINDOW_KEEP_RUNNING_LABEL).size(14),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let settings_btn = button(
            row![
                text(global_constants::MAIN_WINDOW_ICON_SETTINGS).size(16),
                text(global_constants::MAIN_WINDOW_SETTINGS_BUTTON_LABEL).size(14)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([12, 24])
        .style(|theme, status| app_theme::secondary_button_style(theme, status))
        .on_press(OrchestratorMessage::OpenSettings);

        let footer_content = column![system_tray_row, settings_btn]
            .spacing(16)
            .align_x(Alignment::Center);

        let footer_panel = container(footer_content)
            .padding([20, 24])
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: iced::Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                text_color: None,
                snap: false,
            });

        let content = column![
            header_section,
            Space::new().height(Length::Fixed(32.0)),
            action_panel,
            Space::new().height(Length::Fixed(16.0)),
            status_indicator,
            Space::new().height(Length::Fixed(24.0)),
            footer_panel,
        ]
        .spacing(0)
        .padding(32)
        .align_x(Alignment::Center)
        .max_width(500);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(Background::Color(palette.background)),
                    text_color: Some(palette.text),
                    ..Default::default()
                }
            })
            .into()
    }

    fn render_status_indicator(&self) -> Element<'_, OrchestratorMessage> {
        let (status_color, status_icon) = match self.status.as_str() {
            s if s.contains(global_constants::STATUS_KEYWORD_READY) => (
                Color::from_rgb(0.2, 0.8, 0.4),
                global_constants::MAIN_WINDOW_STATUS_ICON_FILLED,
            ),
            s if s.contains(global_constants::STATUS_KEYWORD_LOADING)
                || s.contains(global_constants::STATUS_KEYWORD_INITIALIZING) =>
            {
                (
                    Color::from_rgb(1.0, 0.8, 0.2),
                    global_constants::MAIN_WINDOW_STATUS_ICON_EMPTY,
                )
            }
            s if s.contains(global_constants::STATUS_KEYWORD_ERROR)
                || s.contains(global_constants::STATUS_KEYWORD_FAILED) =>
            {
                (
                    Color::from_rgb(1.0, 0.3, 0.3),
                    global_constants::MAIN_WINDOW_STATUS_ICON_FILLED,
                )
            }
            _ => (
                Color::from_rgba(0.5, 0.5, 0.5, 1.0),
                global_constants::MAIN_WINDOW_STATUS_ICON_FILLED,
            ),
        };

        let status_text = row![
            text(status_icon)
                .size(12)
                .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(status_color),
                }),
            text(&self.status)
                .size(13)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        container(status_text).into()
    }
}
