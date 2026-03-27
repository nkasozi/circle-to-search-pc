use super::*;

const KEYBOARD_SHORTCUT_COPY_TEXT_MACOS: &str = "\u{2318}C";
const KEYBOARD_SHORTCUT_COPY_TEXT_OTHER: &str = "Ctrl+C";
const KEYBOARD_SHORTCUT_SELECT_ALL_MACOS: &str = "\u{2318}A";
const KEYBOARD_SHORTCUT_SELECT_ALL_OTHER: &str = "Ctrl+A";
const SEARCH_INPUT_PLACEHOLDER: &str = "Optional: Add text to refine your search";
const SEARCH_BUTTON_TOOLTIP: &str = "Search Image on Google";
const HELP_HINT_PREFIX: &str = "\u{1f4a1} Click and drag on text to select \u{2022} ";
const HELP_HINT_SUFFIX: &str = " to select all \u{2022} Esc to deselect";

impl InteractiveOcrView {
    pub(super) fn push_copy_text_button<'a>(
        &self,
        mut action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        if self.selected_chars.is_empty() {
            return action_row;
        }

        let copy_shortcut = if cfg!(target_os = "macos") {
            KEYBOARD_SHORTCUT_COPY_TEXT_MACOS
        } else {
            KEYBOARD_SHORTCUT_COPY_TEXT_OTHER
        };
        let copy_text_btn = button(text("📋").size(20))
            .padding([10, 14])
            .style(|_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.4, 0.2, 0.6, 0.9),
                    Color::from_rgba(0.5, 0.3, 0.8, 0.95),
                    Color::from_rgba(0.4, 0.2, 0.7, 0.95),
                    Color::from_rgba(0.6, 0.4, 0.9, 0.6),
                )
            })
            .on_press(InteractiveOcrMessage::CopySelected);
        action_row = action_row.push(
            tooltip(
                copy_text_btn,
                text(format!("Copy Selected Text ({})", copy_shortcut)),
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );
        action_row
    }

    pub(super) fn push_search_controls<'a>(
        &self,
        mut action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let (search_text, is_searching) = match &self.search_state {
            SearchState::Idle => ("🔍", false),
            SearchState::UploadingImage => (Self::spinner_frame_text(self.spinner_frame), true),
            SearchState::Completed => ("✅", true),
            SearchState::Failed(_) => ("❌", true),
        };

        let search_input = text_input("", &self.search_query)
            .on_input(InteractiveOcrMessage::SearchQueryChanged)
            .on_submit(InteractiveOcrMessage::SearchSelected)
            .padding([8, 12])
            .width(Length::Fixed(160.0))
            .align_x(Alignment::Center)
            .style(|_theme: &iced::Theme, _status| text_input::Style {
                background: iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.9)),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.6),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                icon: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
                placeholder: Color::TRANSPARENT,
                value: Color::WHITE,
                selection: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
            });
        action_row = action_row.push(
            tooltip(
                self.build_search_placeholder_overlay(search_input),
                SEARCH_INPUT_PLACEHOLDER,
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );

        let mut search_btn = button(text(search_text).size(20)).padding([10, 14]).style(
            |_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                    Color::from_rgba(0.2, 0.5, 0.9, 0.95),
                    Color::from_rgba(0.1, 0.4, 0.8, 0.95),
                    Color::from_rgba(0.3, 0.6, 1.0, 0.5),
                )
            },
        );
        if !is_searching {
            search_btn = search_btn.on_press(InteractiveOcrMessage::SearchSelected);
        }
        action_row = action_row.push(
            tooltip(search_btn, SEARCH_BUTTON_TOOLTIP, tooltip::Position::Top)
                .style(Self::tooltip_style),
        );
        action_row
    }

    pub(super) fn build_search_placeholder_overlay<'a>(
        &self,
        search_input: iced::widget::TextInput<'a, InteractiveOcrMessage>,
    ) -> Element<'a, InteractiveOcrMessage> {
        let show_placeholder = self.search_query.is_empty();
        let placeholder_overlay = container(
            row![
                text("G").size(14).color(Color::from_rgba(
                    0.259,
                    0.522,
                    0.957,
                    if show_placeholder { 1.0 } else { 0.0 }
                )),
                text("o").size(14).color(Color::from_rgba(
                    0.918,
                    0.263,
                    0.208,
                    if show_placeholder { 1.0 } else { 0.0 }
                )),
                text("o").size(14).color(Color::from_rgba(
                    0.984,
                    0.737,
                    0.02,
                    if show_placeholder { 1.0 } else { 0.0 }
                )),
                text("g").size(14).color(Color::from_rgba(
                    0.259,
                    0.522,
                    0.957,
                    if show_placeholder { 1.0 } else { 0.0 }
                )),
                text("l").size(14).color(Color::from_rgba(
                    0.204,
                    0.659,
                    0.325,
                    if show_placeholder { 1.0 } else { 0.0 }
                )),
                text("e").size(14).color(Color::from_rgba(
                    0.918,
                    0.263,
                    0.208,
                    if show_placeholder { 1.0 } else { 0.0 }
                )),
            ]
            .spacing(0),
        )
        .width(Length::Fixed(160.0))
        .height(Length::Shrink)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .padding([8, 12]);
        stack![search_input, placeholder_overlay].into()
    }

    pub(super) fn build_help_hint(&self) -> Element<'_, InteractiveOcrMessage> {
        let select_all_shortcut = if cfg!(target_os = "macos") {
            KEYBOARD_SHORTCUT_SELECT_ALL_MACOS
        } else {
            KEYBOARD_SHORTCUT_SELECT_ALL_OTHER
        };
        let hint_content = row![
            text(format!(
                "{}{}{}",
                HELP_HINT_PREFIX, select_all_shortcut, HELP_HINT_SUFFIX
            ))
            .size(13)
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(0.9, 0.9, 0.9, 0.95)),
            }),
            button(text("✕").size(12))
                .padding([4, 8])
                .style(|_theme: &iced::Theme, status| {
                    Self::solid_button_style(
                        status,
                        Color::TRANSPARENT,
                        Color::from_rgba(0.4, 0.4, 0.4, 0.8),
                        Color::from_rgba(0.3, 0.3, 0.3, 0.8),
                        Color::TRANSPARENT,
                    )
                })
                .on_press(InteractiveOcrMessage::DismissHelpHint)
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        container(hint_content)
            .padding([10, 16])
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.1, 0.1, 0.15, 0.92,
                ))),
                border: Border {
                    color: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
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
            })
            .into()
    }
}
