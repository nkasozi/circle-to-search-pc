use iced::widget::{button, column, container, image, row, text};
use iced::{Alignment, Element, Length};

use crate::core::models::CaptureBuffer;

pub struct InteractiveOcrView {
    image_handle: iced::widget::image::Handle,
    image_width: u32,
    image_height: u32,
}

#[derive(Debug, Clone)]
pub enum InteractiveOcrMessage {
    Close,
}

impl InteractiveOcrView {
    pub fn build(capture_buffer: CaptureBuffer) -> Self {
        log::info!(
            "[INTERACTIVE_OCR] Creating view for cropped image: {}x{}",
            capture_buffer.width,
            capture_buffer.height
        );

        Self {
            image_handle: capture_buffer.image_handle.clone(),
            image_width: capture_buffer.width,
            image_height: capture_buffer.height,
        }
    }

    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let title = text(format!("Cropped Image ({}x{})", self.image_width, self.image_height))
            .size(24)
            .width(Length::Fill);

        let image_display = image::viewer(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let close_btn = button(text("✖ Close"))
            .padding([8, 16])
            .on_press(InteractiveOcrMessage::Close);

        let buttons = row![close_btn]
            .spacing(10)
            .width(Length::Fill)
            .align_y(Alignment::Center);

        let content = column![title, image_display, buttons]
            .spacing(12)
            .padding(15)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
