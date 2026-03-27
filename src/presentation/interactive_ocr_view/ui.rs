use super::*;
use crate::global_constants;

mod image_actions;
mod search_actions;
mod styles;
mod toolbars;

const OCR_PROMPT_TEXT: &str = "Perform OCR text recognition?";
const OCR_FAILED_PREFIX: &str = "\u{274c} OCR Failed \u{2014} ";
const OCR_RETRY_BUTTON_LABEL: &str = "\u{21ba} Retry OCR";
const TOAST_TEXT_COPIED: &str = "\u{2713} Text copied!";
const TOAST_COPY_TEXT_FAILED: &str = "\u{2717} Copy failed";
const TOAST_IMAGE_COPIED: &str = "\u{2713} Image copied!";
const TOAST_COPY_IMAGE_FAILED_PREFIX: &str = "\u{2717} Copy failed: ";
const TOAST_SAVE_SUCCESS_PREFIX: &str = "\u{2713} Saved to ";
const TOAST_SAVE_FAILED_PREFIX: &str = "\u{2717} Save failed: ";

impl InteractiveOcrView {
    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let image_with_overlay = self.render_image_with_overlay();
        let image_layer = container(image_with_overlay)
            .width(Length::Fill)
            .height(Length::Fill);
        let mut layers: Vec<Element<'_, InteractiveOcrMessage>> = vec![image_layer.into()];

        layers.push(self.build_status_banner().into());

        if let Some(toast) = self.build_copy_toast() {
            layers.push(self.position_top_centered(toast, 60.0));
        }
        if let Some(toast) = self.build_image_copy_toast() {
            layers.push(self.position_top_centered(toast, 60.0));
        }
        if let Some(toast) = self.build_save_state_toast() {
            layers.push(self.position_top_centered(toast, 100.0));
        }

        if self.show_help_hint && !self.char_positions.is_empty() {
            let hint_positioned = container(self.build_help_hint())
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 80.0,
                    left: 0.0,
                })
                .align_x(Alignment::Center)
                .align_y(Alignment::End);
            layers.push(hint_positioned.into());
        }

        layers.push(self.build_draw_toolbar().into());
        layers.push(self.build_action_toolbar().into());

        container(stack(layers))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08))),
                ..Default::default()
            })
            .into()
    }

    fn build_status_banner(&self) -> Element<'_, InteractiveOcrMessage> {
        let status_text = self.build_status_text();
        let banner_inner_content: Element<'_, InteractiveOcrMessage> =
            if matches!(self.ocr_state, OcrState::Idle) {
                let prompt_label =
                    text(OCR_PROMPT_TEXT)
                        .size(14)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        });
                let confirm_btn =
                    button(
                        text("✓")
                            .size(14)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::WHITE),
                            }),
                    )
                    .padding([2, 10])
                    .style(|_theme: &iced::Theme, status| {
                        Self::solid_button_style(
                            status,
                            Color::from_rgba(0.1, 0.55, 0.15, 0.9),
                            Color::from_rgba(0.1, 0.7, 0.2, 0.95),
                            Color::from_rgba(0.05, 0.5, 0.1, 0.95),
                            Color::from_rgba(0.3, 0.9, 0.4, 0.6),
                        )
                    })
                    .on_press(InteractiveOcrMessage::StartOcr);
                row![prompt_label, confirm_btn]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
            } else if matches!(self.ocr_state, OcrState::Processing) {
                let label = text(global_constants::STATUS_PROCESSING_OCR)
                    .size(14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    });
                let cancel_btn =
                    button(
                        text("✕")
                            .size(13)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::WHITE),
                            }),
                    )
                    .padding([2, 8])
                    .style(|_theme: &iced::Theme, status| {
                        Self::solid_button_style(
                            status,
                            Color::from_rgba(0.5, 0.1, 0.1, 0.8),
                            Color::from_rgba(0.8, 0.2, 0.2, 0.9),
                            Color::from_rgba(0.6, 0.1, 0.1, 0.9),
                            Color::from_rgba(0.9, 0.3, 0.3, 0.5),
                        )
                    })
                    .on_press(InteractiveOcrMessage::CancelOcr);
                row![label, cancel_btn]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
            } else if let OcrState::Failed(ref ocr_error) = self.ocr_state {
                let error_label = text(format!("{}{}", OCR_FAILED_PREFIX, ocr_error))
                    .size(14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(1.0, 0.5, 0.5)),
                    });
                let retry_btn = button(text(OCR_RETRY_BUTTON_LABEL).size(13).style(|_theme| {
                    iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    }
                }))
                .padding([2, 8])
                .style(|_theme: &iced::Theme, status| {
                    Self::solid_button_style(
                        status,
                        Color::from_rgba(0.1, 0.45, 0.1, 0.85),
                        Color::from_rgba(0.1, 0.6, 0.1, 0.9),
                        Color::from_rgba(0.1, 0.4, 0.1, 0.9),
                        Color::from_rgba(0.3, 0.8, 0.3, 0.5),
                    )
                })
                .on_press(InteractiveOcrMessage::RetryOcr);
                row![error_label, retry_btn]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
            } else if matches!(self.ocr_state, OcrState::Completed)
                && self.selected_chars.is_empty()
            {
                let completed_label =
                    text(status_text)
                        .size(14)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        });
                let retry_ocr_btn =
                    button(
                        text("↺")
                            .size(14)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(0.7, 0.7, 0.7, 0.9)),
                            }),
                    )
                    .padding([2, 6])
                    .style(|_theme: &iced::Theme, status| {
                        Self::solid_button_style(
                            status,
                            Color::from_rgba(0.15, 0.15, 0.15, 0.0),
                            Color::from_rgba(0.3, 0.3, 0.3, 0.9),
                            Color::from_rgba(0.2, 0.2, 0.2, 0.9),
                            Color::from_rgba(0.4, 0.4, 0.4, 0.4),
                        )
                    })
                    .on_press(InteractiveOcrMessage::RetryOcr);
                let clear_ocr_btn =
                    button(
                        text("✕")
                            .size(13)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(0.7, 0.7, 0.7, 0.9)),
                            }),
                    )
                    .padding([2, 6])
                    .style(|_theme: &iced::Theme, status| {
                        Self::solid_button_style(
                            status,
                            Color::from_rgba(0.15, 0.15, 0.15, 0.0),
                            Color::from_rgba(0.5, 0.1, 0.1, 0.9),
                            Color::from_rgba(0.4, 0.05, 0.05, 0.9),
                            Color::from_rgba(0.4, 0.4, 0.4, 0.4),
                        )
                    })
                    .on_press(InteractiveOcrMessage::ClearOcrOverlay);
                row![completed_label, retry_ocr_btn, clear_ocr_btn]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
            } else {
                text(status_text)
                    .size(14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    })
                    .into()
            };

        container(
            container(banner_inner_content)
                .padding([8, 16])
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.1, 0.1, 0.1, 0.8,
                    ))),
                    border: Border {
                        color: Color::from_rgba(0.3, 0.6, 1.0, 0.6),
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
                }),
        )
        .width(Length::Fill)
        .padding(iced::Padding {
            top: 16.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .align_x(Alignment::Center)
        .into()
    }

    fn build_copy_toast(&self) -> Option<Element<'_, InteractiveOcrMessage>> {
        match &self.copy_state {
            CopyState::Success => {
                Some(self.build_toast(TOAST_TEXT_COPIED, Color::from_rgb(0.2, 0.8, 0.4)))
            }
            CopyState::Failed => {
                Some(self.build_toast(TOAST_COPY_TEXT_FAILED, Color::from_rgb(0.9, 0.3, 0.3)))
            }
            CopyState::Idle => None,
        }
    }

    fn build_image_copy_toast(&self) -> Option<Element<'_, InteractiveOcrMessage>> {
        match &self.image_copy_state {
            ImageCopyState::Success => Some(Self::build_save_toast(
                TOAST_IMAGE_COPIED.to_string(),
                Color::from_rgb(0.2, 0.8, 0.4),
            )),
            ImageCopyState::Failed(error) => Some(Self::build_save_toast(
                format!("{}{}", TOAST_COPY_IMAGE_FAILED_PREFIX, error),
                Color::from_rgb(0.9, 0.3, 0.3),
            )),
            ImageCopyState::Idle | ImageCopyState::Preparing | ImageCopyState::Copying => None,
        }
    }

    fn build_save_state_toast(&self) -> Option<Element<'_, InteractiveOcrMessage>> {
        match &self.save_state {
            SaveState::Success(path) => Some(Self::build_save_toast(
                format!("{}{}", TOAST_SAVE_SUCCESS_PREFIX, path),
                Color::from_rgb(0.2, 0.8, 0.4),
            )),
            SaveState::Failed(error) => Some(Self::build_save_toast(
                format!("{}{}", TOAST_SAVE_FAILED_PREFIX, error),
                Color::from_rgb(0.9, 0.3, 0.3),
            )),
            SaveState::Idle | SaveState::Preparing | SaveState::Saving => None,
        }
    }

    fn position_top_centered<'a>(
        &self,
        element: Element<'a, InteractiveOcrMessage>,
        top: f32,
    ) -> Element<'a, InteractiveOcrMessage> {
        container(element)
            .width(Length::Fill)
            .padding(iced::Padding {
                top,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .into()
    }
}
