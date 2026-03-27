use super::*;
use iced::widget::{button, canvas, container, image, row, stack, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

impl CaptureView {
    pub fn render_ui(&self) -> Element<'_, CaptureViewMessage> {
        let screenshot_viewer = image::viewer(self.capture_buffer.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);
        let overlay_canvas = canvas(self).width(Length::Fill).height(Length::Fill);
        let mut layers: Vec<Element<'_, CaptureViewMessage>> =
            vec![screenshot_viewer.into(), overlay_canvas.into()];

        if !self.is_selecting {
            layers.push(self.build_status_banner().into());
            layers.push(self.build_toolbar().into());
        }

        container(stack(layers))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn build_status_banner(&self) -> Element<'_, CaptureViewMessage> {
        let status_message = if self.calculate_selection_rectangle().is_some() {
            "Press Enter to confirm selection or draw a new region"
        } else {
            match self.draw_mode {
                DrawMode::Rectangle => "Click and drag to select a region",
                DrawMode::Freeform => "Click and drag to draw a freeform shape",
            }
        };

        container(
            container(text(status_message).size(16).style(|_theme| text::Style {
                color: Some(Color::WHITE),
            }))
            .padding([12, 24])
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.85))),
                border: Border {
                    color: Color::from_rgba(0.3, 0.6, 1.0, 0.8),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                text_color: None,
                snap: false,
            }),
        )
        .width(Length::Fill)
        .padding(iced::Padding {
            top: 80.0,
            right: 20.0,
            bottom: 0.0,
            left: 0.0,
        })
        .align_x(Alignment::End)
        .into()
    }

    fn build_toolbar(&self) -> Element<'_, CaptureViewMessage> {
        let rect_btn = button(text("⬜ Rectangle"))
            .padding([8, 16])
            .style(move |theme: &iced::Theme, status| {
                self.toolbar_button_style(theme, status, self.draw_mode == DrawMode::Rectangle)
            })
            .on_press(CaptureViewMessage::SetDrawMode(DrawMode::Rectangle));
        let freeform_btn = button(text("✏️ Freeform"))
            .padding([8, 16])
            .style(move |theme: &iced::Theme, status| {
                self.toolbar_button_style(theme, status, self.draw_mode == DrawMode::Freeform)
            })
            .on_press(CaptureViewMessage::SetDrawMode(DrawMode::Freeform));
        let window_btn = button(text("🪟 Window"))
            .padding([8, 16])
            .style(move |theme: &iced::Theme, status| {
                self.toolbar_button_style(theme, status, false)
            })
            .on_press(CaptureViewMessage::SelectWindow);

        let toolbar = container(
            row![rect_btn, freeform_btn, window_btn]
                .spacing(8)
                .padding(8),
        )
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.85))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.9),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            text_color: None,
            snap: false,
        });

        container(toolbar)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 80.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .into()
    }

    fn toolbar_button_style(
        &self,
        _theme: &iced::Theme,
        status: button::Status,
        is_active: bool,
    ) -> button::Style {
        let base_color = if is_active {
            Color::from_rgb(0.3, 0.6, 1.0)
        } else {
            Color::from_rgba(0.4, 0.4, 0.4, 0.8)
        };
        match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(base_color)),
                text_color: Color::WHITE,
                border: Border {
                    color: if is_active {
                        Color::from_rgb(0.4, 0.7, 1.0)
                    } else {
                        Color::from_rgba(0.5, 0.5, 0.5, 0.9)
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(if is_active {
                    Color::from_rgb(0.4, 0.7, 1.0)
                } else {
                    Color::from_rgba(0.5, 0.5, 0.5, 0.9)
                })),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::from_rgb(0.5, 0.8, 1.0),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(if is_active {
                    Color::from_rgb(0.2, 0.5, 0.9)
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 0.9)
                })),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::from_rgb(0.3, 0.6, 0.9),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.5))),
                text_color: Color::from_rgba(0.6, 0.6, 0.6, 1.0),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
        }
    }
}
