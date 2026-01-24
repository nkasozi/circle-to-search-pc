use iced::widget::{button, column, container, image, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Shadow, Vector};

use crate::core::models::WindowInfo;

pub struct WindowPickerView {
    windows: Vec<WindowInfo>,
    selected_window_id: Option<u32>,
    is_loading: bool,
    spinner_frame: usize,
    filter_query: String,
}

#[derive(Debug, Clone)]
pub enum WindowPickerMessage {
    WindowSelected(u32),
    ConfirmSelection,
    Cancel,
    RefreshWindows,
    CaptureFullScreen,
    SpinnerTick,
    FilterChanged(String),
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
            spinner_frame: 0,
            filter_query: String::new(),
        }
    }

    pub fn set_windows(&mut self, windows: Vec<WindowInfo>) {
        self.windows = windows;
        self.is_loading = false;
    }

    pub fn set_loading(&mut self, is_loading: bool) {
        self.is_loading = is_loading;
    }

    #[allow(dead_code)]
    pub fn get_selected_window_id(&self) -> Option<u32> {
        self.selected_window_id
    }

    pub fn get_selected_window_info(&self) -> Option<&WindowInfo> {
        self.selected_window_id
            .and_then(|id| self.windows.iter().find(|w| w.id == id))
    }

    fn get_filtered_windows(&self) -> Vec<&WindowInfo> {
        if self.filter_query.is_empty() {
            return self.windows.iter().collect();
        }
        let query_lower = self.filter_query.to_lowercase();
        self.windows
            .iter()
            .filter(|w| {
                w.app_name.to_lowercase().contains(&query_lower)
                    || w.title.to_lowercase().contains(&query_lower)
            })
            .collect()
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
                self.spinner_frame = 0;
                self.filter_query.clear();
            }
            WindowPickerMessage::CaptureFullScreen => {
                log::info!("[WINDOW_PICKER] Capture full screen selected");
            }
            WindowPickerMessage::SpinnerTick => {
                if self.is_loading {
                    self.spinner_frame = (self.spinner_frame + 1) % 8;
                }
            }
            WindowPickerMessage::FilterChanged(query) => {
                log::debug!("[WINDOW_PICKER] Filter changed: {}", query);
                self.filter_query = query;
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
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
            let spinner = spinner_chars[self.spinner_frame];

            container(
                column![
                    text(spinner).size(32).style(|_theme: &iced::Theme| {
                        iced::widget::text::Style {
                            color: Some(Color::from_rgba(0.4, 0.7, 1.0, 1.0)),
                        }
                    }),
                    text("Loading windows...")
                        .size(16)
                        .style(|_theme: &iced::Theme| iced::widget::text::Style {
                            color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                        }),
                ]
                .spacing(12)
                .align_x(Alignment::Center),
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
            let filtered_windows = self.get_filtered_windows();
            if filtered_windows.is_empty() {
                container(
                    text("No matching windows")
                        .size(16)
                        .style(|_theme: &iced::Theme| iced::widget::text::Style {
                            color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                        }),
                )
                .padding(40)
                .center_x(Length::Fill)
                .into()
            } else {
                let window_items: Vec<Element<'_, WindowPickerMessage>> = filtered_windows
                    .iter()
                    .map(|window| self.render_window_item(window))
                    .collect();

                scrollable(column(window_items).spacing(8).padding(4))
                    .height(Length::FillPortion(1))
                    .into()
            }
        };

        let filter_input = text_input("Filter apps...", &self.filter_query)
            .on_input(WindowPickerMessage::FilterChanged)
            .padding([10, 14])
            .width(Length::Fill)
            .style(|_theme: &iced::Theme, _status| text_input::Style {
                background: iced::Background::Color(Color::from_rgba(0.15, 0.15, 0.18, 0.9)),
                border: Border {
                    color: Color::from_rgba(0.3, 0.3, 0.35, 0.6),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                icon: Color::from_rgba(0.5, 0.5, 0.5, 0.8),
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.8),
                value: Color::WHITE,
                selection: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
            });

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

        let filter_row = row![text("🔍").size(16), filter_input,]
            .spacing(8)
            .align_y(Alignment::Center);

        let content = column![
            header,
            full_screen_btn,
            text("Or select a specific window:")
                .size(14)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
            filter_row,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_window(id: u32, app_name: &str, title: &str) -> WindowInfo {
        WindowInfo {
            id,
            app_name: app_name.to_string(),
            title: title.to_string(),
            width: 800,
            height: 600,
            is_minimized: false,
            thumbnail: None,
        }
    }

    #[test]
    fn test_build_creates_view_with_windows() {
        let windows = vec![
            create_test_window(1, "Chrome", "Google"),
            create_test_window(2, "Firefox", "Mozilla"),
        ];
        let view = WindowPickerView::build(windows);

        assert_eq!(view.windows.len(), 2);
        assert!(view.selected_window_id.is_none());
        assert!(!view.is_loading);
        assert!(view.filter_query.is_empty());
    }

    #[test]
    fn test_build_creates_empty_view() {
        let view = WindowPickerView::build(vec![]);

        assert!(view.windows.is_empty());
    }

    #[test]
    fn test_set_windows_updates_list() {
        let mut view = WindowPickerView::build(vec![]);
        let windows = vec![create_test_window(1, "App", "Window")];

        view.set_windows(windows);

        assert_eq!(view.windows.len(), 1);
        assert!(!view.is_loading);
    }

    #[test]
    fn test_set_loading_updates_state() {
        let mut view = WindowPickerView::build(vec![]);

        view.set_loading(true);
        assert!(view.is_loading);

        view.set_loading(false);
        assert!(!view.is_loading);
    }

    #[test]
    fn test_update_window_selected_sets_id() {
        let windows = vec![create_test_window(42, "App", "Window")];
        let mut view = WindowPickerView::build(windows);

        view.update(WindowPickerMessage::WindowSelected(42));

        assert_eq!(view.selected_window_id, Some(42));
    }

    #[test]
    fn test_update_refresh_windows_sets_loading() {
        let mut view = WindowPickerView::build(vec![]);
        view.filter_query = "test".to_string();

        view.update(WindowPickerMessage::RefreshWindows);

        assert!(view.is_loading);
        assert_eq!(view.spinner_frame, 0);
        assert!(view.filter_query.is_empty());
    }

    #[test]
    fn test_update_spinner_tick_increments_frame() {
        let mut view = WindowPickerView::build(vec![]);
        view.is_loading = true;

        view.update(WindowPickerMessage::SpinnerTick);
        assert_eq!(view.spinner_frame, 1);

        view.update(WindowPickerMessage::SpinnerTick);
        assert_eq!(view.spinner_frame, 2);
    }

    #[test]
    fn test_update_spinner_tick_wraps_around() {
        let mut view = WindowPickerView::build(vec![]);
        view.is_loading = true;
        view.spinner_frame = 7;

        view.update(WindowPickerMessage::SpinnerTick);

        assert_eq!(view.spinner_frame, 0);
    }

    #[test]
    fn test_update_spinner_tick_does_nothing_when_not_loading() {
        let mut view = WindowPickerView::build(vec![]);
        view.is_loading = false;
        view.spinner_frame = 3;

        view.update(WindowPickerMessage::SpinnerTick);

        assert_eq!(view.spinner_frame, 3);
    }

    #[test]
    fn test_update_filter_changed_updates_query() {
        let mut view = WindowPickerView::build(vec![]);

        view.update(WindowPickerMessage::FilterChanged("chrome".to_string()));

        assert_eq!(view.filter_query, "chrome");
    }

    #[test]
    fn test_get_filtered_windows_returns_all_when_empty_query() {
        let windows = vec![
            create_test_window(1, "Chrome", "Google"),
            create_test_window(2, "Firefox", "Mozilla"),
        ];
        let view = WindowPickerView::build(windows);

        let filtered = view.get_filtered_windows();

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_get_filtered_windows_filters_by_app_name() {
        let windows = vec![
            create_test_window(1, "Chrome", "Google"),
            create_test_window(2, "Firefox", "Mozilla"),
            create_test_window(3, "Chrome", "GitHub"),
        ];
        let mut view = WindowPickerView::build(windows);
        view.filter_query = "chrome".to_string();

        let filtered = view.get_filtered_windows();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|w| w.app_name == "Chrome"));
    }

    #[test]
    fn test_get_filtered_windows_filters_by_title() {
        let windows = vec![
            create_test_window(1, "Chrome", "Google Search"),
            create_test_window(2, "Firefox", "Google Maps"),
            create_test_window(3, "Safari", "Apple"),
        ];
        let mut view = WindowPickerView::build(windows);
        view.filter_query = "google".to_string();

        let filtered = view.get_filtered_windows();

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_get_filtered_windows_case_insensitive() {
        let windows = vec![
            create_test_window(1, "CHROME", "Google"),
            create_test_window(2, "chrome", "Test"),
            create_test_window(3, "Firefox", "Chrome Page"),
        ];
        let mut view = WindowPickerView::build(windows);
        view.filter_query = "CHROME".to_string();

        let filtered = view.get_filtered_windows();

        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_get_filtered_windows_returns_empty_when_no_match() {
        let windows = vec![
            create_test_window(1, "Chrome", "Google"),
            create_test_window(2, "Firefox", "Mozilla"),
        ];
        let mut view = WindowPickerView::build(windows);
        view.filter_query = "safari".to_string();

        let filtered = view.get_filtered_windows();

        assert!(filtered.is_empty());
    }

    #[test]
    fn test_get_selected_window_id_returns_none_initially() {
        let view = WindowPickerView::build(vec![]);

        assert!(view.get_selected_window_id().is_none());
    }

    #[test]
    fn test_get_selected_window_id_returns_selected() {
        let windows = vec![create_test_window(99, "App", "Window")];
        let mut view = WindowPickerView::build(windows);
        view.update(WindowPickerMessage::WindowSelected(99));

        assert_eq!(view.get_selected_window_id(), Some(99));
    }

    #[test]
    fn test_get_selected_window_info_returns_none_when_not_selected() {
        let windows = vec![create_test_window(1, "App", "Window")];
        let view = WindowPickerView::build(windows);

        assert!(view.get_selected_window_info().is_none());
    }

    #[test]
    fn test_get_selected_window_info_returns_window() {
        let windows = vec![
            create_test_window(1, "Chrome", "Google"),
            create_test_window(2, "Firefox", "Mozilla"),
        ];
        let mut view = WindowPickerView::build(windows);
        view.update(WindowPickerMessage::WindowSelected(2));

        let info = view.get_selected_window_info();

        assert!(info.is_some());
        assert_eq!(info.unwrap().app_name, "Firefox");
    }

    #[test]
    fn test_get_selected_window_info_returns_none_for_invalid_id() {
        let windows = vec![create_test_window(1, "App", "Window")];
        let mut view = WindowPickerView::build(windows);
        view.selected_window_id = Some(999);

        assert!(view.get_selected_window_info().is_none());
    }
}
