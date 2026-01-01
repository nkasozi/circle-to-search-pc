use iced::widget::{button, column, container, image, row, text, canvas, stack};
use iced::{Alignment, Element, Length, Point, Rectangle, Size, Color};

use crate::core::models::{CaptureBuffer, OcrResult};

pub struct InteractiveOcrView {
    image_handle: iced::widget::image::Handle,
    image_width: u32,
    image_height: u32,
    ocr_result: Option<OcrResult>,
    selected_text_indices: Vec<usize>,
    hover_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum InteractiveOcrMessage {
    Close,
    TextBlockClicked(usize),
    CopySelected,
    SearchSelected,
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
            selected_text_indices: Vec::new(),
            hover_index: None,
        }
    }

    pub fn set_ocr_result(&mut self, result: OcrResult) {
        log::info!(
            "[INTERACTIVE_OCR] Setting OCR result with {} text blocks",
            result.text_blocks.len()
        );
        self.ocr_result = Some(result);
    }

    pub fn update(&mut self, message: InteractiveOcrMessage) {
        match message {
            InteractiveOcrMessage::TextBlockClicked(index) => {
                log::debug!("[INTERACTIVE_OCR] Text block {} clicked", index);
                if self.selected_text_indices.contains(&index) {
                    self.selected_text_indices.retain(|&i| i != index);
                } else {
                    self.selected_text_indices.push(index);
                }
            }
            InteractiveOcrMessage::CopySelected => {
                if let Some(ref result) = self.ocr_result {
                    let selected_text: String = self.selected_text_indices
                        .iter()
                        .filter_map(|&i| result.text_blocks.get(i))
                        .map(|block| block.content.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if !selected_text.is_empty() {
                        log::info!("[INTERACTIVE_OCR] Copying text: {}", selected_text);
                        if let Err(e) = arboard::Clipboard::new()
                            .and_then(|mut clipboard| clipboard.set_text(&selected_text)) {
                            log::error!("[INTERACTIVE_OCR] Failed to copy to clipboard: {}", e);
                        } else {
                            log::info!("[INTERACTIVE_OCR] Text copied to clipboard");
                        }
                    }
                }
            }
            InteractiveOcrMessage::SearchSelected => {
                if let Some(ref result) = self.ocr_result {
                    let selected_text: String = self.selected_text_indices
                        .iter()
                        .filter_map(|&i| result.text_blocks.get(i))
                        .map(|block| block.content.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if !selected_text.is_empty() {
                        log::info!("[INTERACTIVE_OCR] Searching for: {}", selected_text);
                        let search_url = format!("https://www.google.com/search?q={}",
                            urlencoding::encode(&selected_text));

                        if let Err(e) = open::that(&search_url) {
                            log::error!("[INTERACTIVE_OCR] Failed to open browser: {}", e);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let status_text = if let Some(ref result) = self.ocr_result {
            if self.selected_text_indices.is_empty() {
                format!("Detected {} text blocks - Click to select", result.text_blocks.len())
            } else {
                format!("Selected {} words", self.selected_text_indices.len())
            }
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

        let mut button_row = row![]
            .spacing(10)
            .align_y(Alignment::Center);

        if !self.selected_text_indices.is_empty() {
            let copy_btn = button(text("📋 Copy"))
                .padding([8, 16])
                .on_press(InteractiveOcrMessage::CopySelected);

            let search_btn = button(text("🔍 Search"))
                .padding([8, 16])
                .on_press(InteractiveOcrMessage::SearchSelected);

            button_row = button_row.push(copy_btn).push(search_btn);
        }

        let close_btn = button(text("✖ Close"))
            .padding([8, 16])
            .on_press(InteractiveOcrMessage::Close);

        button_row = button_row.push(close_btn);

        let buttons = button_row.width(Length::Fill);

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
            selected_indices: self.selected_text_indices.clone(),
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
    selected_indices: Vec<usize>,
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

        let img_width = self.image_width as f32;
        let img_height = self.image_height as f32;
        let img_aspect = img_width / img_height;
        let bounds_aspect = bounds.width / bounds.height;

        let (display_width, display_height, offset_x, offset_y) = if img_aspect > bounds_aspect {
            let display_width = bounds.width;
            let display_height = bounds.width / img_aspect;
            let offset_y = (bounds.height - display_height) / 2.0;
            (display_width, display_height, 0.0, offset_y)
        } else {
            let display_height = bounds.height;
            let display_width = bounds.height * img_aspect;
            let offset_x = (bounds.width - display_width) / 2.0;
            (display_width, display_height, offset_x, 0.0)
        };

        let scale_x = display_width / img_width;
        let scale_y = display_height / img_height;

        for (idx, text_block) in self.text_blocks.iter().enumerate() {
            let rect_bounds = text_block.bounds;

            let scaled_x = offset_x + (rect_bounds.x * scale_x);
            let scaled_y = offset_y + (rect_bounds.y * scale_y);
            let scaled_width = rect_bounds.width * scale_x;
            let scaled_height = rect_bounds.height * scale_y;

            let is_selected = self.selected_indices.contains(&idx);

            let (fill_color, stroke_color) = if is_selected {
                (Color::from_rgba(0.3, 0.8, 0.3, 0.3), Color::from_rgb(0.2, 0.9, 0.2))
            } else {
                (Color::from_rgba(0.2, 0.6, 1.0, 0.2), Color::from_rgb(0.3, 0.7, 1.0))
            };

            let rect_path = canvas::Path::rectangle(
                Point::new(scaled_x, scaled_y),
                Size::new(scaled_width, scaled_height)
            );

            frame.fill_rectangle(
                Point::new(scaled_x, scaled_y),
                Size::new(scaled_width, scaled_height),
                fill_color
            );

            frame.stroke(
                &rect_path,
                canvas::Stroke::default()
                    .with_color(stroke_color)
                    .with_width(2.0)
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::default()
        }
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<InteractiveOcrMessage>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = event {
            if let Some(cursor_position) = cursor.position_in(bounds) {
                let img_width = self.image_width as f32;
                let img_height = self.image_height as f32;
                let img_aspect = img_width / img_height;
                let bounds_aspect = bounds.width / bounds.height;

                let (display_width, display_height, offset_x, offset_y) = if img_aspect > bounds_aspect {
                    let display_width = bounds.width;
                    let display_height = bounds.width / img_aspect;
                    let offset_y = (bounds.height - display_height) / 2.0;
                    (display_width, display_height, 0.0, offset_y)
                } else {
                    let display_height = bounds.height;
                    let display_width = bounds.height * img_aspect;
                    let offset_x = (bounds.width - display_width) / 2.0;
                    (display_width, display_height, offset_x, 0.0)
                };

                let scale_x = display_width / img_width;
                let scale_y = display_height / img_height;

                for (idx, text_block) in self.text_blocks.iter().enumerate() {
                    let rect_bounds = text_block.bounds;
                    let scaled_x = offset_x + (rect_bounds.x * scale_x);
                    let scaled_y = offset_y + (rect_bounds.y * scale_y);
                    let scaled_width = rect_bounds.width * scale_x;
                    let scaled_height = rect_bounds.height * scale_y;

                    let text_rect = Rectangle::new(
                        Point::new(scaled_x, scaled_y),
                        Size::new(scaled_width, scaled_height)
                    );

                    if text_rect.contains(cursor_position) {
                        log::debug!("[OCR_OVERLAY] Clicked on text block {}: '{}'", idx, text_block.content);
                        return Some(canvas::Action::publish(InteractiveOcrMessage::TextBlockClicked(idx)));
                    }
                }
            }
        }

        None
    }
}
