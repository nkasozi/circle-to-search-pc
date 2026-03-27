use super::*;

impl InteractiveOcrView {
    pub(super) fn build_toast<'a>(
        &self,
        message: &'a str,
        color: Color,
    ) -> Element<'a, InteractiveOcrMessage> {
        container(
            text(message)
                .size(14)
                .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(color),
                }),
        )
        .padding([8, 16])
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.1, 0.1, 0.1, 0.9,
            ))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            text_color: None,
            snap: false,
        })
        .into()
    }

    pub(super) fn build_save_toast(
        message: String,
        color: Color,
    ) -> Element<'static, InteractiveOcrMessage> {
        container(
            text(message)
                .size(14)
                .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(color),
                }),
        )
        .padding([8, 16])
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.1, 0.1, 0.1, 0.9,
            ))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            text_color: None,
            snap: false,
        })
        .into()
    }

    pub(super) fn floating_btn_style(
        &self,
        status: button::Status,
        is_active: bool,
    ) -> button::Style {
        let inactive_border = Color::from_rgba(0.5, 0.5, 0.5, 0.4);
        let active_border = Color::from_rgba(0.4, 0.7, 1.0, 0.7);
        let base_background = if is_active {
            Color::from_rgba(0.3, 0.6, 1.0, 0.9)
        } else {
            Color::from_rgba(0.15, 0.15, 0.15, 0.85)
        };
        let hovered_background = if is_active {
            Color::from_rgba(0.4, 0.7, 1.0, 0.95)
        } else {
            Color::from_rgba(0.25, 0.25, 0.25, 0.9)
        };
        let pressed_background = if is_active {
            Color::from_rgba(0.2, 0.5, 0.9, 0.95)
        } else {
            Color::from_rgba(0.2, 0.2, 0.2, 0.9)
        };
        let border_color = if is_active {
            active_border
        } else {
            inactive_border
        };
        Self::solid_button_style(
            status,
            base_background,
            hovered_background,
            pressed_background,
            border_color,
        )
    }

    pub(super) fn color_btn_style(
        &self,
        status: button::Status,
        is_selected: bool,
    ) -> button::Style {
        let border_color = if is_selected {
            Color::from_rgba(1.0, 1.0, 1.0, 0.8)
        } else {
            Color::from_rgba(0.4, 0.4, 0.4, 0.4)
        };
        let border_width = if is_selected { 2.0 } else { 1.0 };
        let background = match status {
            button::Status::Hovered => Color::from_rgba(0.3, 0.3, 0.3, 0.9),
            button::Status::Pressed => Color::from_rgba(0.2, 0.2, 0.2, 0.9),
            _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
        };
        button::Style {
            background: Some(iced::Background::Color(background)),
            text_color: Color::WHITE,
            border: Border {
                color: border_color,
                width: border_width,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub(super) fn tooltip_style(_theme: &iced::Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.1, 0.1, 0.1, 0.95,
            ))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.6),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            text_color: Some(Color::WHITE),
            snap: false,
        }
    }

    pub(super) fn render_image_with_overlay(&self) -> Element<'_, InteractiveOcrMessage> {
        let image_viewer = image::viewer(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);
        let ocr_overlay = OcrOverlay {
            char_positions: self.char_positions.clone(),
            image_width: self.image_width,
            image_height: self.image_height,
            selected_indices: self.selected_chars.clone(),
            draw_strokes: self.draw_strokes.clone(),
            current_stroke_points: self.current_stroke_points.clone(),
            is_drawing: self.is_drawing,
            draw_color: self.draw_color,
            draw_width: self.draw_width,
            draw_mode_enabled: self.draw_mode_enabled,
            draw_panel_position: self.draw_panel_position,
            draw_panel_is_dragging: self.draw_panel_is_dragging,
        };
        let overlay_canvas =
            container(canvas(ocr_overlay).width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill);
        stack![image_viewer, overlay_canvas]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub(super) fn solid_button_style(
        status: button::Status,
        base_background: Color,
        hovered_background: Color,
        pressed_background: Color,
        border_color: Color,
    ) -> button::Style {
        let background = match status {
            button::Status::Hovered => hovered_background,
            button::Status::Pressed => pressed_background,
            _ => base_background,
        };
        button::Style {
            background: Some(iced::Background::Color(background)),
            text_color: Color::WHITE,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub(super) fn spinner_frame_text(spinner_frame: usize) -> &'static str {
        const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        SPINNER_FRAMES[spinner_frame]
    }
}
