use super::*;
use crate::presentation::design::surfaces::ToastKind;
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

mod image_actions;
mod search_actions;
mod styles;
mod toolbars;

const READING_TEXT_LABEL: &str = "Reading text…";
const OCR_FAILED_PREFIX: &str = "Couldn't read text: ";
const TOAST_TEXT_COPIED: &str = "Text copied";
const TOAST_COPY_TEXT_FAILED: &str = "Couldn't copy text";
const TOAST_IMAGE_COPIED: &str = "Image copied";
const TOAST_COPY_IMAGE_FAILED_PREFIX: &str = "Couldn't copy image: ";
const TOAST_SAVE_SUCCESS_PREFIX: &str = "Saved to ";
const TOAST_SAVE_FAILED_PREFIX: &str = "Couldn't save: ";
const TOAST_TOP_OFFSET: f32 = 64.0;
const BANNER_TOP_OFFSET: f32 = 16.0;
const HELP_HINT_BOTTOM_OFFSET: f32 = 76.0;
const IMAGE_BACKDROP: Color = Color::from_rgb(0.06, 0.065, 0.08);

impl InteractiveOcrView {
    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let image_layer = container(self.render_image_with_overlay())
            .width(Length::Fill)
            .height(Length::Fill);
        let mut layers: Vec<Element<'_, InteractiveOcrMessage>> = vec![image_layer.into()];
        if let Some(banner) = self.build_status_banner() {
            layers.push(self.position_top_centered(banner, BANNER_TOP_OFFSET));
        }
        if let Some(toast) = self.build_active_toast() {
            layers.push(self.position_top_centered(toast, TOAST_TOP_OFFSET));
        }
        if self.show_help_hint && !self.char_positions.is_empty() {
            layers.push(position_bottom_centered(
                self.build_help_hint(),
                HELP_HINT_BOTTOM_OFFSET,
            ));
        }
        layers.push(self.build_draw_toolbar());
        layers.push(self.build_action_toolbar());
        container(stack(layers))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(IMAGE_BACKDROP)),
                ..Default::default()
            })
            .into()
    }

    fn build_status_banner(&self) -> Option<Element<'_, InteractiveOcrMessage>> {
        let banner_items: Vec<Element<'_, InteractiveOcrMessage>> = match &self.ocr_state {
            OcrState::Idle => return None,
            OcrState::Processing => self.build_processing_banner_items(),
            OcrState::Failed(reason) => build_failed_banner_items(reason),
            OcrState::Completed => self.build_completed_banner_items(),
        };
        let banner_row = iced::widget::Row::with_children(banner_items)
            .spacing(spacing::SMALL)
            .align_y(Alignment::Center);
        Some(
            container(banner_row)
                .padding([spacing::EXTRA_SMALL + 2.0, spacing::MEDIUM])
                .style(surfaces::hud_pill)
                .into(),
        )
    }

    fn build_processing_banner_items(&self) -> Vec<Element<'_, InteractiveOcrMessage>> {
        vec![
            banner_text(Self::spinner_frame_text(self.spinner_frame).to_string()),
            banner_text(READING_TEXT_LABEL.to_string()),
            banner_icon_button(Icon::Close, InteractiveOcrMessage::CancelOcr),
        ]
    }

    fn build_completed_banner_items(&self) -> Vec<Element<'_, InteractiveOcrMessage>> {
        vec![
            icon(
                Icon::ScanText,
                icon_size::SMALL,
                IconTint::Fixed(HUD_TOKENS.accent_hover),
            )
            .into(),
            banner_text(self.build_status_text()),
            banner_icon_button(Icon::Refresh, InteractiveOcrMessage::RetryOcr),
            banner_icon_button(Icon::Close, InteractiveOcrMessage::ClearOcrOverlay),
        ]
    }

    fn build_active_toast(&self) -> Option<Element<'_, InteractiveOcrMessage>> {
        let save_toast = match &self.save_state {
            SaveState::Success(path) => Some((
                format!("{}{}", TOAST_SAVE_SUCCESS_PREFIX, path),
                ToastKind::Success,
            )),
            SaveState::Failed(error) => Some((
                format!("{}{}", TOAST_SAVE_FAILED_PREFIX, error),
                ToastKind::Error,
            )),
            _ => None,
        };
        let image_toast = match &self.image_copy_state {
            ImageCopyState::Success => Some((TOAST_IMAGE_COPIED.to_string(), ToastKind::Success)),
            ImageCopyState::Failed(error) => Some((
                format!("{}{}", TOAST_COPY_IMAGE_FAILED_PREFIX, error),
                ToastKind::Error,
            )),
            _ => None,
        };
        let text_toast = match &self.copy_state {
            CopyState::Success => Some((TOAST_TEXT_COPIED.to_string(), ToastKind::Success)),
            CopyState::Failed => Some((TOAST_COPY_TEXT_FAILED.to_string(), ToastKind::Error)),
            CopyState::Idle => None,
        };
        save_toast
            .or(image_toast)
            .or(text_toast)
            .map(|(message, kind)| Self::build_toast_with_kind(message, kind))
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
                ..iced::Padding::ZERO
            })
            .align_x(Alignment::Center)
            .into()
    }
}

fn build_failed_banner_items(reason: &str) -> Vec<Element<'static, InteractiveOcrMessage>> {
    vec![
        icon(Icon::Close, icon_size::SMALL, IconTint::Danger).into(),
        banner_text(format!("{}{}", OCR_FAILED_PREFIX, reason)),
        banner_icon_button(Icon::Refresh, InteractiveOcrMessage::RetryOcr),
    ]
}

fn banner_text(content: String) -> Element<'static, InteractiveOcrMessage> {
    text(content)
        .size(font_size::BODY)
        .style(surfaces::hud_text)
        .into()
}

fn banner_icon_button(
    button_icon: Icon,
    message: InteractiveOcrMessage,
) -> Element<'static, InteractiveOcrMessage> {
    button(icon(
        button_icon,
        icon_size::SMALL,
        IconTint::Fixed(HUD_TOKENS.text_secondary),
    ))
    .padding(spacing::EXTRA_SMALL)
    .style(button_style(ButtonTone::HudIcon))
    .on_press(message)
    .into()
}

fn position_bottom_centered(
    element: Element<'_, InteractiveOcrMessage>,
    bottom: f32,
) -> Element<'_, InteractiveOcrMessage> {
    container(element)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            bottom,
            ..iced::Padding::ZERO
        })
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .into()
}
