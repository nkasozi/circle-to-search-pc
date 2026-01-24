use iced::widget::{button, column, container, image, row, scrollable, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Shadow, Vector};

use crate::core::models::WindowInfo;

pub struct WindowPickerView {
    windows: Vec<WindowInfo>,
    selected_window_id: Option<u32>,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub enum WindowPickerMessage {
    WindowSelected(u32),
    ConfirmSelection,
    Cancel,
    RefreshWindows,
    CaptureFullScreen,
}

impl WindowPickerView {
    pub fn build(windows: Vec<WindowInfo>) -> Self {
        log::info!(
            "[WINDOW_PICKER] Creating view with {} windows",
            windows.len()
        );
        Self {
            windows,
            selected_window_id: None,
            is_loading: false,
        }
    }

    pub fn set_windows(&mut self, windows: Vec<WindowInfo>) {
        self.windows = windows;
        self.is_loading = false;
    }

    pub fn set_loading(&mut self, is_loading: bool) {
        self.is_loading = is_loading;
    }

    pub fn get_selected_window_id(&self) -> Option<u32> {
        self.selected_window_id
    }

    pub fn update(&mut self, message: WindowPickerMessage) {
        match message {
            WindowPickerMessage::WindowSelected(id) => {
                log::debug!("[WINDOW_PICKER] Window selected: {}", id);
                self.selected_window_id = Some(id);
            }
            WindowPickerMessage::ConfirmSelection => {
                log::info!("[WINDOW_PICKER] Selection confirmed");
            }
            WindowPickerMessage::Cancel => {
                log::info!("[WINDOW_PICKER] Selection cancelled");
            }
            WindowPickerMessage::RefreshWindows => {
                log::info!("[WINDOW_PICKER] Refreshing window list");
                self.is_loading = true;
            }
            WindowPickerMessage::CaptureFullScreen => {
                log::info!("[WINDOW_PICKER] Capture full screen selected");
            }
        }
    }

    pub fn render_ui(&self) -> Element<'_, WindowPickerMessage> {
        let title = text("Select Window to Capture")
            .size(24)
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::WHITE),
            });

        let subtitle = text("Choose a window or capture the full screen")
            .size(14)
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(0.7, 0.7, 0.7, 1.0)),
            });

        let header = column![title, subtitle]
            .spacing(8)
            .align_x(Alignment::Center);

        let full_screen_btn = button(
            row![
                text("🖥️").size(24),
                column![
                    text("Full Screen").size(16),
                    text("Capture entire screen")
                        .size(12)
                        .style(|_theme: &iced::Theme| iced::widget::text::Style {
                            color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                        }),
                ]
                .spacing(2)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([16, 20])
        .style(|_theme: &iced::Theme, status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(0.2, 0.4, 0.6, 0.9),
                button::Status::Pressed => Color::from_rgba(0.15, 0.35, 0.55, 0.9),
                _ => Color::from_rgba(0.15, 0.15, 0.2, 0.9),
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::from_rgba(0.3, 0.5, 0.7, 0.6),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .on_press(WindowPickerMessage::CaptureFullScreen);

        let window_list: Element<'_, WindowPickerMessage> = if self.is_loading {
            container(
                text("Loading windows...")
                    .size(16)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                    }),
            )
            .padding(40)
            .center_x(Length::Fill)
            .into()
        } else if self.windows.is_empty() {
            container(
                text("No windows found")
                    .size(16)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                    }),
            )
            .padding(40)
            .center_x(Length::Fill)
            .into()
        } else {
            let window_items: Vec<Element<'_, WindowPickerMessage>> = self
                .windows
                .iter()
                .map(|window| self.render_window_item(window))
                .collect();

            scrollable(column(window_items).spacing(8).padding(4))
                .height(Length::FillPortion(1))
                .into()
        };

        let mut bottom_row = row![].spacing(12);

        let refresh_btn = button(text("🔄 Refresh").size(14))
            .padding([10, 16])
            .style(|_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.3, 0.3, 0.3, 0.9),
                    button::Status::Pressed => Color::from_rgba(0.2, 0.2, 0.2, 0.9),
                    _ => Color::from_rgba(0.2, 0.2, 0.2, 0.8),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .on_press(WindowPickerMessage::RefreshWindows);

        let cancel_btn = button(text("Cancel").size(14))
            .padding([10, 20])
            .style(|_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.5, 0.2, 0.2, 0.9),
                    button::Status::Pressed => Color::from_rgba(0.4, 0.15, 0.15, 0.9),
                    _ => Color::from_rgba(0.3, 0.15, 0.15, 0.8),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.5, 0.3, 0.3, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .on_press(WindowPickerMessage::Cancel);

        bottom_row = bottom_row.push(refresh_btn);
        bottom_row = bottom_row.push(Space::new().width(Length::Fill));
        bottom_row = bottom_row.push(cancel_btn);

        if self.selected_window_id.is_some() {
            let confirm_btn = button(text("Capture Selected").size(14))
                .padding([10, 20])
                .style(|_theme: &iced::Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => Color::from_rgba(0.2, 0.6, 0.3, 0.95),
                        button::Status::Pressed => Color::from_rgba(0.15, 0.5, 0.25, 0.95),
                        _ => Color::from_rgba(0.15, 0.5, 0.2, 0.9),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: Color::WHITE,
                        border: Border {
                            color: Color::from_rgba(0.3, 0.7, 0.4, 0.6),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    }
                })
                .on_press(WindowPickerMessage::ConfirmSelection);
            bottom_row = bottom_row.push(confirm_btn);
        }

        let content = column![
            header,
            full_screen_btn,
            text("Or select a specific window:")
                .size(14)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
            window_list,
            bottom_row,
        ]
        .spacing(16)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: None,
                snap: false,
            })
            .into()
    }

    fn render_window_item(&self, window: &WindowInfo) -> Element<'_, WindowPickerMessage> {
        let is_selected = self.selected_window_id == Some(window.id);

        let thumbnail_element: Element<'_, WindowPickerMessage> = match &window.thumbnail {
            Some(handle) => image(handle.clone())
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(60.0))
                .into(),
            None => container(text("📷").size(24).style(|_theme: &iced::Theme| {
                iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.4, 0.4, 0.4, 1.0)),
                }
            }))
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(60.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.15, 0.15, 0.15, 1.0,
                ))),
                border: Border {
                    color: Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow::default(),
                text_color: None,
                snap: false,
            })
            .into(),
        };

        let minimized_badge: Element<'_, WindowPickerMessage> = if window.is_minimized {
            text("(minimized)")
                .size(11)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.8, 0.6, 0.2, 1.0)),
                })
                .into()
        } else {
            text("").into()
        };

        let window_info = column![
            text(window.display_name())
                .size(14)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::WHITE),
                }),
            row![
                text(format!("{}x{}", window.width, window.height))
                    .size(11)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
                    }),
                minimized_badge,
            ]
            .spacing(8),
        ]
        .spacing(4);

        let content = row![thumbnail_element, window_info]
            .spacing(12)
            .align_y(Alignment::Center);

        let window_id = window.id;
        button(content)
            .width(Length::Fill)
            .padding([12, 16])
            .style(move |_theme: &iced::Theme, status| {
                let bg = if is_selected {
                    Color::from_rgba(0.2, 0.4, 0.6, 0.9)
                } else {
                    match status {
                        button::Status::Hovered => Color::from_rgba(0.2, 0.2, 0.25, 0.9),
                        button::Status::Pressed => Color::from_rgba(0.15, 0.15, 0.2, 0.9),
                        _ => Color::from_rgba(0.15, 0.15, 0.18, 0.8),
                    }
                };
                let border_color = if is_selected {
                    Color::from_rgba(0.3, 0.6, 0.9, 0.8)
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 0.4)
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: border_color,
                        width: if is_selected { 2.0 } else { 1.0 },
                        radius: 8.0.into(),
                    },
                    shadow: if is_selected {
                        Shadow {
                            color: Color::from_rgba(0.2, 0.4, 0.6, 0.3),
                            offset: Vector::new(0.0, 2.0),
                            blur_radius: 8.0,
                        }
                    } else {
                        Shadow::default()
                    },
                    snap: false,
                }
            })
            .on_press(WindowPickerMessage::WindowSelected(window_id))
            .into()
    }
}
