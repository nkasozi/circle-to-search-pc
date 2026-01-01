use iced::widget::{button, column, container, image, row, text, canvas, stack};
use iced::{Alignment, Element, Length, Point, Rectangle, Size, Color};

use crate::core::models::{CaptureBuffer, OcrResult};

pub struct InteractiveOcrView {
    image_handle: iced::widget::image::Handle,
    image_width: u32,
    image_height: u32,
    ocr_result: Option<OcrResult>,
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
            ocr_result: None,
        }
    }

    pub fn set_ocr_result(&mut self, result: OcrResult) {
        log::info!(
            "[INTERACTIVE_OCR] Setting OCR result with {} text blocks",
            result.text_blocks.len()
        );
        self.ocr_result = Some(result);
    }

    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let status_text = if let Some(ref result) = self.ocr_result {
            format!("Detected {} text blocks", result.text_blocks.len())
        } else {
            "Processing OCR...".to_string()
        };

        let title = text(status_text)
            .size(20)
            .width(Length::Fill);

        let image_with_overlay = if let Some(ref ocr_result) = self.ocr_result {
            self.render_image_with_overlay(ocr_result)
        } else {
            image::viewer(self.image_handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let close_btn = button(text("✖ Close"))
            .padding([8, 16])
            .on_press(InteractiveOcrMessage::Close);

        let buttons = row![close_btn]
            .spacing(10)
            .width(Length::Fill)
            .align_y(Alignment::Center);

        let content = column![title, image_with_overlay, buttons]
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

    fn render_image_with_overlay(&self, ocr_result: &OcrResult) -> Element<'_, InteractiveOcrMessage> {
        let image_viewer = image::viewer(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let ocr_overlay = OcrOverlay {
            text_blocks: ocr_result.text_blocks.clone(),
            image_width: self.image_width,
            image_height: self.image_height,
        };

        let overlay_canvas = container(
            canvas(ocr_overlay)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill);

        stack![
            image_viewer,
            overlay_canvas
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

struct OcrOverlay {
    text_blocks: Vec<crate::core::models::DetectedText>,
    image_width: u32,
    image_height: u32,
}

impl canvas::Program<InteractiveOcrMessage> for OcrOverlay {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let scale_x = bounds.width / self.image_width as f32;
        let scale_y = bounds.height / self.image_height as f32;

        log::debug!(
            "[OCR_OVERLAY] Drawing {} text blocks with scale {}x{}, bounds: {:?}",
            self.text_blocks.len(),
            scale_x,
            scale_y,
            bounds
        );

        for (idx, text_block) in self.text_blocks.iter().enumerate() {
            let rect_bounds = text_block.bounds;

            let scaled_x = rect_bounds.x * scale_x;
            let scaled_y = rect_bounds.y * scale_y;
            let scaled_width = rect_bounds.width * scale_x;
            let scaled_height = rect_bounds.height * scale_y;

            log::debug!(
                "[OCR_OVERLAY] Block {}: '{}' at ({:.1},{:.1}) {:.1}x{:.1}",
                idx,
                text_block.content.chars().take(20).collect::<String>(),
                scaled_x,
                scaled_y,
                scaled_width,
                scaled_height
            );

            let rect_path = canvas::Path::rectangle(
                Point::new(scaled_x, scaled_y),
                Size::new(scaled_width, scaled_height)
            );

            frame.fill_rectangle(
                Point::new(scaled_x, scaled_y),
                Size::new(scaled_width, scaled_height),
                Color::from_rgba(0.2, 0.6, 1.0, 0.2)
            );

            frame.stroke(
                &rect_path,
                canvas::Stroke::default()
                    .with_color(Color::from_rgb(0.3, 0.7, 1.0))
                    .with_width(2.0)
            );
        }

        vec![frame.into_geometry()]
    }
}
