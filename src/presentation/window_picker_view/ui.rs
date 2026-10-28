use iced::widget::{
    button, column, container, image, row, scrollable, text, text_input, Column, Space,
};
use iced::{Alignment, Element, Length};

use super::{WindowPickerMessage, WindowPickerView};
use crate::core::models::WindowInfo;
use crate::presentation::design::components::labeled_button;
use crate::presentation::design::inputs::field_style;
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

mod constants;

use constants::*;

impl WindowPickerView {
    pub fn render_ui(&self) -> Element<'_, WindowPickerMessage> {
        let header = column![
            text(TITLE).size(font_size::TITLE),
            text(SUBTITLE)
                .size(font_size::CAPTION)
                .style(surfaces::secondary_text),
        ]
        .spacing(spacing::EXTRA_SMALL);
        let filter = row![
            icon(Icon::Search, icon_size::SMALL + 2.0, IconTint::TextMuted),
            text_input(FILTER_PLACEHOLDER, &self.filter_query)
                .on_input(WindowPickerMessage::FilterChanged)
                .padding([spacing::SMALL, spacing::MEDIUM])
                .size(font_size::BODY)
                .style(field_style),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center);
        let content = column![
            header,
            render_full_screen_card(),
            text(WINDOWS_HEADING).size(font_size::HEADING),
            filter,
            container(self.render_window_list()).height(Length::Fill),
            self.render_footer(),
        ]
        .spacing(spacing::LARGE)
        .padding(spacing::EXTRA_LARGE);
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(surfaces::window_background)
            .into()
    }

    fn render_window_list(&self) -> Element<'_, WindowPickerMessage> {
        let filtered_windows = self.get_filtered_windows();
        let placeholder = match (
            self.is_loading,
            self.windows.is_empty(),
            filtered_windows.is_empty(),
        ) {
            (true, _, _) => Some(format!(
                "{}  {}",
                SPINNER_FRAMES[self.spinner_frame % SPINNER_FRAMES.len()],
                LOADING_LABEL
            )),
            (false, true, _) => Some(NO_WINDOWS_LABEL.to_string()),
            (false, false, true) => Some(NO_MATCHES_LABEL.to_string()),
            _ => None,
        };
        if let Some(message) = placeholder {
            return container(
                text(message)
                    .size(font_size::BODY)
                    .style(surfaces::muted_text),
            )
            .padding(spacing::HUGE)
            .center_x(Length::Fill)
            .into();
        }
        let items: Vec<Element<'_, WindowPickerMessage>> = filtered_windows
            .into_iter()
            .map(|window| self.render_window_item(window))
            .collect();
        scrollable(
            Column::with_children(items)
                .spacing(spacing::EXTRA_SMALL)
                .padding(iced::Padding {
                    right: spacing::MEDIUM,
                    ..iced::Padding::ZERO
                }),
        )
        .height(Length::Fill)
        .into()
    }

    fn render_window_item<'a>(
        &'a self,
        window: &'a WindowInfo,
    ) -> Element<'a, WindowPickerMessage> {
        let thumbnail = render_window_thumbnail(window);
        let size_label = format!("{} × {}", window.width, window.height);
        let detail = match window.is_minimized {
            true => format!("{}  ·  {}", size_label, MINIMIZED_LABEL),
            false => size_label,
        };
        let details = column![
            text(window.display_name()).size(font_size::BODY),
            text(detail)
                .size(font_size::SMALL)
                .style(surfaces::muted_text)
        ]
        .spacing(2.0);
        let tone = match self.selected_window_id == Some(window.id) {
            true => ButtonTone::SegmentActive,
            false => ButtonTone::ListRow,
        };
        button(
            row![thumbnail, details]
                .spacing(spacing::MEDIUM)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(spacing::SMALL)
        .style(button_style(tone))
        .on_press(WindowPickerMessage::WindowSelected(window.id))
        .into()
    }

    fn render_footer(&self) -> Element<'_, WindowPickerMessage> {
        let mut footer = row![
            labeled_button(
                Icon::Refresh,
                REFRESH_LABEL,
                WindowPickerMessage::RefreshWindows,
                ButtonTone::Ghost,
                IconTint::TextSecondary
            ),
            Space::new().width(Length::Fill),
            labeled_button(
                Icon::Close,
                CANCEL_LABEL,
                WindowPickerMessage::Cancel,
                ButtonTone::Ghost,
                IconTint::TextSecondary
            ),
        ]
        .spacing(spacing::SMALL)
        .align_y(Alignment::Center);
        if self.selected_window_id.is_some() {
            footer = footer.push(labeled_button(
                Icon::Check,
                CAPTURE_LABEL,
                WindowPickerMessage::ConfirmSelection,
                ButtonTone::Primary,
                iced::Color::WHITE,
            ));
        }
        footer.into()
    }
}

fn render_full_screen_card<'a>() -> Element<'a, WindowPickerMessage> {
    let content = row![
        icon(Icon::Monitor, icon_size::LARGE, IconTint::Accent),
        column![
            text(FULL_SCREEN_TITLE).size(font_size::HEADING),
            text(FULL_SCREEN_DESCRIPTION)
                .size(font_size::CAPTION)
                .style(surfaces::secondary_text)
        ]
        .spacing(2.0),
    ]
    .spacing(spacing::MEDIUM)
    .align_y(Alignment::Center);
    button(content)
        .width(Length::Fill)
        .padding(spacing::LARGE)
        .style(button_style(ButtonTone::Card))
        .on_press(WindowPickerMessage::CaptureFullScreen)
        .into()
}

fn render_window_thumbnail(window: &WindowInfo) -> Element<'_, WindowPickerMessage> {
    match &window.thumbnail {
        Some(handle) => image(handle.clone())
            .width(Length::Fixed(THUMBNAIL_WIDTH))
            .height(Length::Fixed(THUMBNAIL_HEIGHT))
            .border_radius(crate::presentation::design::tokens::radius::STANDARD)
            .into(),
        None => container(icon(Icon::AppWindow, icon_size::LARGE, IconTint::TextMuted))
            .width(Length::Fixed(THUMBNAIL_WIDTH))
            .height(Length::Fixed(THUMBNAIL_HEIGHT))
            .center_x(Length::Fixed(THUMBNAIL_WIDTH))
            .center_y(Length::Fixed(THUMBNAIL_HEIGHT))
            .style(surfaces::raised_card)
            .into(),
    }
}
