use super::*;

const TOOLBAR_DRAW_DISABLE_LABEL: &str = "Disable Draw Mode";
const TOOLBAR_DRAW_ENABLE_LABEL: &str = "Enable Draw Mode";
const TOOLBAR_CLEAR_DRAWINGS_LABEL: &str = "Clear Drawings";
const TOOLBAR_POSITION_BOTTOM_LABEL: &str = "Move toolbar to bottom";
const TOOLBAR_POSITION_TOP_LABEL: &str = "Move toolbar to top";
const TOOLBAR_DRAG_HANDLE_TEXT: &str = "⠿  drag  ⠿";

impl InteractiveOcrView {
    pub(super) fn build_draw_toolbar(&self) -> Element<'_, InteractiveOcrMessage> {
        let mut draw_row = row![].spacing(6).align_y(Alignment::Center);
        let draw_toggle = button(text("🖊️").size(16))
            .padding([8, 12])
            .style(move |_theme: &iced::Theme, status| {
                self.floating_btn_style(status, self.draw_mode_enabled)
            })
            .on_press(InteractiveOcrMessage::ToggleDrawMode);
        let draw_tooltip_text = if self.draw_mode_enabled {
            TOOLBAR_DRAW_DISABLE_LABEL
        } else {
            TOOLBAR_DRAW_ENABLE_LABEL
        };
        draw_row = draw_row.push(
            tooltip(draw_toggle, draw_tooltip_text, tooltip::Position::Bottom)
                .style(Self::tooltip_style),
        );

        if self.draw_mode_enabled {
            for color in [
                Color::from_rgb(1.0, 0.2, 0.2),
                Color::from_rgb(0.2, 0.6, 1.0),
                Color::from_rgb(0.2, 0.8, 0.2),
                Color::from_rgb(1.0, 0.85, 0.0),
            ] {
                let is_selected = (self.draw_color.r - color.r).abs() < 0.1
                    && (self.draw_color.g - color.g).abs() < 0.1
                    && (self.draw_color.b - color.b).abs() < 0.1;
                let color_btn = button(text("●").size(18).style(move |_theme: &iced::Theme| {
                    iced::widget::text::Style { color: Some(color) }
                }))
                .padding([6, 10])
                .style(move |_theme: &iced::Theme, status| {
                    self.color_btn_style(status, is_selected)
                })
                .on_press(InteractiveOcrMessage::SetDrawColor(color));
                draw_row = draw_row.push(color_btn);
            }

            let clear_btn = button(text("🗑").size(14))
                .padding([8, 10])
                .style(|_theme: &iced::Theme, status| {
                    Self::solid_button_style(
                        status,
                        Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                        Color::from_rgba(0.8, 0.2, 0.2, 0.9),
                        Color::from_rgba(0.6, 0.1, 0.1, 0.9),
                        Color::from_rgba(0.5, 0.5, 0.5, 0.4),
                    )
                })
                .on_press(InteractiveOcrMessage::ClearDrawings);
            draw_row = draw_row.push(
                tooltip(
                    clear_btn,
                    TOOLBAR_CLEAR_DRAWINGS_LABEL,
                    tooltip::Position::Bottom,
                )
                .style(Self::tooltip_style),
            );
        }

        let draw_handle_strip =
            container(text(TOOLBAR_DRAG_HANDLE_TEXT).size(11).style(|_theme| {
                iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.55, 0.55, 0.55, 0.8)),
                }
            }))
            .padding([5, 10])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.07, 0.07, 0.07, 0.6,
                ))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius {
                        top_left: 8.0,
                        top_right: 8.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                },
                shadow: Shadow::default(),
                text_color: None,
                snap: false,
            });
        let draw_panel_body = container(draw_row)
            .padding([6, 10])
            .width(Length::Shrink)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.1, 0.1, 0.1, 0.85,
                ))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_left: 8.0,
                        bottom_right: 8.0,
                    },
                },
                shadow: Shadow::default(),
                text_color: None,
                snap: false,
            });
        let draw_toolbar =
            container(iced::widget::column![draw_handle_strip, draw_panel_body].spacing(0))
                .width(Length::Shrink)
                .style(|_theme| iced::widget::container::Style {
                    background: None,
                    border: Border {
                        color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    text_color: None,
                    snap: false,
                });

        container(draw_toolbar)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: self.draw_panel_position.y,
                left: self.draw_panel_position.x,
                right: 0.0,
                bottom: 0.0,
            })
            .align_x(Alignment::Start)
            .align_y(Alignment::Start)
            .into()
    }

    pub(super) fn build_action_toolbar(&self) -> Element<'_, InteractiveOcrMessage> {
        let mut action_row = row![].spacing(6).align_y(Alignment::Center);
        let position_icon = if self.toolbar_offset.y > 50.0 {
            "↓"
        } else {
            "↑"
        };
        let position_tooltip = if self.toolbar_offset.y > 50.0 {
            TOOLBAR_POSITION_BOTTOM_LABEL
        } else {
            TOOLBAR_POSITION_TOP_LABEL
        };
        let toggle_position_btn = button(text(position_icon).size(16))
            .padding([8, 10])
            .style(|_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.15, 0.15, 0.15, 0.7),
                    Color::from_rgba(0.3, 0.3, 0.3, 0.95),
                    Color::from_rgba(0.2, 0.2, 0.2, 0.95),
                    Color::from_rgba(0.4, 0.4, 0.4, 0.4),
                )
            })
            .on_press(InteractiveOcrMessage::ToggleToolbarPosition);
        action_row = action_row.push(
            tooltip(
                toggle_position_btn,
                position_tooltip,
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );

        action_row = self.push_copy_text_button(action_row);
        action_row = self.push_search_controls(action_row);
        action_row = self.push_copy_image_button(action_row);
        action_row = self.push_save_button(action_row);
        action_row = self.push_recrop_button(action_row);
        action_row = self.push_close_button(action_row);

        let action_toolbar =
            container(action_row)
                .padding([6, 10])
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.1, 0.1, 0.1, 0.85,
                    ))),
                    border: Border {
                        color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    text_color: None,
                    snap: false,
                });

        let is_at_top = self.toolbar_offset.y > 50.0;
        let vertical_alignment = if is_at_top {
            Alignment::Start
        } else {
            Alignment::End
        };
        let top_padding = if is_at_top { 60.0 } else { 0.0 };
        let bottom_padding = if is_at_top { 0.0 } else { 16.0 };

        container(action_toolbar)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: top_padding,
                right: 0.0,
                bottom: bottom_padding,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .align_y(vertical_alignment)
            .into()
    }
}
