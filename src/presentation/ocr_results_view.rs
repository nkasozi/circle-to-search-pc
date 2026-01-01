use iced::widget::{button, column, container, image, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

pub struct OcrResultsView {
    image_handle: iced::widget::image::Handle,
    extracted_text: String,
    selected_text: String,
}

#[derive(Debug, Clone)]
pub enum OcrResultsMessage {
    CopyText,
    GoogleSearch,
    TextSelected(String),
    Close,
}

impl OcrResultsView {
    pub fn build_with_results(
        image_handle: iced::widget::image::Handle,
        extracted_text: String,
    ) -> Self {
        log::info!(
            "[OCR_VIEW] Creating view with {} characters of text",
            extracted_text.len()
        );
        Self {
            image_handle,
            extracted_text: extracted_text.clone(),
            selected_text: String::new(),
        }
    }

    pub fn update(&mut self, message: OcrResultsMessage) {
        match message {
            OcrResultsMessage::TextSelected(text) => {
                log::debug!("[OCR_VIEW] Text selected: {}", text);
                self.selected_text = text;
            }
            OcrResultsMessage::CopyText => {
                log::info!("[OCR_VIEW] Copying text to clipboard");
            }
            OcrResultsMessage::GoogleSearch => {
                log::info!("[OCR_VIEW] Opening Google search");
            }
            OcrResultsMessage::Close => {
                log::info!("[OCR_VIEW] Closing results view");
            }
        }
    }

    pub fn render_ui(&self) -> Element<'_, OcrResultsMessage> {
        let title = text("OCR Results").size(28).width(Length::Fill);

        let image_preview = image::viewer(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::FillPortion(2));

        let text_display = text_input("Extracted text will appear here...", &self.extracted_text)
            .size(16)
            .width(Length::Fill);

        let copy_btn = button(text("📋 Copy Text"))
            .padding([10, 20])
            .on_press(OcrResultsMessage::CopyText);

        let search_btn = button(text("🔍 Google Search"))
            .padding([10, 20])
            .on_press(OcrResultsMessage::GoogleSearch);

        let close_btn = button(text("✖ Close"))
            .padding([10, 20])
            .on_press(OcrResultsMessage::Close);

        let buttons = row![copy_btn, search_btn, close_btn]
            .spacing(10)
            .width(Length::Fill);

        let text_section = column![
            text("Extracted Text:").size(18),
            scrollable(text_display).height(Length::FillPortion(1)),
        ]
        .spacing(10)
        .width(Length::Fill);

        let content = column![title, image_preview, text_section, buttons,]
            .spacing(20)
            .padding(20)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn get_extracted_text(&self) -> &str {
        &self.extracted_text
    }
}
