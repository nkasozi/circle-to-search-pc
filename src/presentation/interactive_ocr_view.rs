use iced::widget::{button, column, container, image, row, text, canvas, stack};
use iced::{Alignment, Element, Length, Point, Rectangle, Size, Color};

use crate::core::models::{CaptureBuffer, OcrResult};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Idle,
    UploadingImage,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharPosition {
    pub word_index: usize,
    pub char_index: usize,
    pub bounds: Rectangle,
    pub character: char,
}

pub struct InteractiveOcrView {
    image_handle: iced::widget::image::Handle,
    image_width: u32,
    image_height: u32,
    capture_buffer: CaptureBuffer,
    ocr_result: Option<OcrResult>,
    char_positions: Vec<CharPosition>,
    selected_chars: Vec<usize>,
    drag_start: Option<usize>,
    is_selecting: bool,
    search_state: SearchState,
    theme_mode: crate::user_settings::ThemeMode,
    show_toast: bool,
}

#[derive(Debug, Clone)]
pub enum InteractiveOcrMessage {
    Close,
    StartDrag(usize),
    UpdateDrag(usize),
    EndDrag,
    CopySelected,
    SearchSelected,
    SearchUploading,
    SearchCompleted,
    SearchFailed(String),
    HideToast,
}

impl InteractiveOcrView {
    pub fn build(capture_buffer: CaptureBuffer, theme_mode: crate::user_settings::ThemeMode) -> Self {
        log::info!(
            "[INTERACTIVE_OCR] Creating view for cropped image: {}x{}",
            capture_buffer.width,
            capture_buffer.height
        );

        Self {
            image_handle: capture_buffer.image_handle.clone(),
            image_width: capture_buffer.width,
            image_height: capture_buffer.height,
            capture_buffer,
            ocr_result: None,
            char_positions: Vec::new(),
            selected_chars: Vec::new(),
            drag_start: None,
            is_selecting: false,
            search_state: SearchState::Idle,
            theme_mode,
            show_toast: false,
        }
    }

    pub fn get_capture_buffer(&self) -> &CaptureBuffer {
        &self.capture_buffer
    }

    pub fn set_ocr_result(&mut self, result: OcrResult) {
        log::info!(
            "[INTERACTIVE_OCR] Setting OCR result with {} text blocks",
            result.text_blocks.len()
        );

        self.char_positions = Self::calculate_char_positions(&result);
        log::info!("[INTERACTIVE_OCR] Calculated {} character positions", self.char_positions.len());
        self.ocr_result = Some(result);
    }

    fn calculate_char_positions(result: &OcrResult) -> Vec<CharPosition> {
        let mut positions = Vec::new();

        for (word_idx, word) in result.text_blocks.iter().enumerate() {
            let char_count = word.content.chars().count();
            if char_count == 0 {
                continue;
            }

            let char_width = word.bounds.width / char_count as f32;

            for (char_idx, ch) in word.content.chars().enumerate() {
                let char_x = word.bounds.x + (char_idx as f32 * char_width);
                positions.push(CharPosition {
                    word_index: word_idx,
                    char_index: char_idx,
                    bounds: Rectangle {
                        x: char_x,
                        y: word.bounds.y,
                        width: char_width,
                        height: word.bounds.height,
                    },
                    character: ch,
                });
            }
        }

        positions
    }

    pub fn update(&mut self, message: InteractiveOcrMessage) {
        match message {
            InteractiveOcrMessage::Close => {
            }
            InteractiveOcrMessage::StartDrag(char_idx) => {
                if !self.is_selecting {
                    log::debug!("[INTERACTIVE_OCR] Starting new selection at char {}", char_idx);
                    self.drag_start = Some(char_idx);
                    self.is_selecting = true;
                } else {
                    log::debug!("[INTERACTIVE_OCR] Ending current drag session, keeping selections");
                    self.is_selecting = false;
                    self.drag_start = None;
                }
            }
            InteractiveOcrMessage::UpdateDrag(char_idx) => {
                if self.is_selecting {
                    if let Some(start_idx) = self.drag_start {
                        let min_idx = start_idx.min(char_idx);
                        let max_idx = start_idx.max(char_idx);
                        let new_selection: Vec<usize> = (min_idx..=max_idx).collect();

                        let mut combined_selection = self.selected_chars.clone();
                        for idx in new_selection {
                            if !combined_selection.contains(&idx) {
                                combined_selection.push(idx);
                            }
                        }
                        combined_selection.sort_unstable();
                        self.selected_chars = combined_selection;
                    }
                }
            }
            InteractiveOcrMessage::EndDrag => {
                log::debug!("[INTERACTIVE_OCR] Drag ended with {} chars selected", self.selected_chars.len());
            }
            InteractiveOcrMessage::CopySelected => {
                let selected_text = self.get_selected_text_with_layout();
                if !selected_text.is_empty() {
                    log::info!("[INTERACTIVE_OCR] Copying text: {}", selected_text);
                    if let Err(e) = arboard::Clipboard::new()
                        .and_then(|mut clipboard| clipboard.set_text(&selected_text)) {
                        log::error!("[INTERACTIVE_OCR] Failed to copy to clipboard: {}", e);
                    } else {
                        log::info!("[INTERACTIVE_OCR] Text copied to clipboard");
                        self.show_toast = true;
                    }
                }
            }
            InteractiveOcrMessage::SearchSelected => {
                if matches!(self.search_state, SearchState::Idle) {
                    log::info!("[INTERACTIVE_OCR] Starting reverse image search");
                    self.search_state = SearchState::UploadingImage;
                }
            }
            InteractiveOcrMessage::SearchUploading => {
                log::debug!("[INTERACTIVE_OCR] Search state: Uploading image");
                self.search_state = SearchState::UploadingImage;
            }
            InteractiveOcrMessage::SearchCompleted => {
                log::info!("[INTERACTIVE_OCR] Search completed successfully");
                self.search_state = SearchState::Completed;
                self.search_state = SearchState::Idle;
            }
            InteractiveOcrMessage::SearchFailed(error) => {
                log::error!("[INTERACTIVE_OCR] Search failed: {}", error);
                self.search_state = SearchState::Failed(error.clone());
                self.search_state = SearchState::Idle;
            }
            InteractiveOcrMessage::HideToast => {
                self.show_toast = false;
            }
        }
    }

    fn get_selected_text_with_layout(&self) -> String {
        if self.selected_chars.is_empty() {
            return String::new();
        }

        let mut selected_positions: Vec<&CharPosition> = self.selected_chars
            .iter()
            .filter_map(|&idx| self.char_positions.get(idx))
            .collect();

        if selected_positions.is_empty() {
            return String::new();
        }

        selected_positions.sort_by(|a, b| {
            let y_diff = (a.bounds.y - b.bounds.y).abs();
            if y_diff > 10.0 {
                a.bounds.y.partial_cmp(&b.bounds.y).unwrap()
            } else {
                a.bounds.x.partial_cmp(&b.bounds.x).unwrap()
            }
        });

        let is_vertical = self.detect_vertical_layout(&selected_positions);

        let mut result = String::new();
        let mut last_y = selected_positions[0].bounds.y;
        let mut last_word_idx = selected_positions[0].word_index;

        for pos in selected_positions {
            if is_vertical && (pos.bounds.y - last_y).abs() > 10.0 {
                result.push('\n');
                last_y = pos.bounds.y;
                last_word_idx = pos.word_index;
            } else if pos.word_index != last_word_idx {
                result.push(' ');
                last_word_idx = pos.word_index;
            }
            result.push(pos.character);
        }

        result
    }

    fn detect_vertical_layout(&self, positions: &[&CharPosition]) -> bool {
        if positions.len() < 2 {
            return false;
        }

        let mut y_changes = 0;
        for i in 1..positions.len() {
            if (positions[i].bounds.y - positions[i-1].bounds.y).abs() > 10.0 {
                y_changes += 1;
            }
        }

        y_changes as f32 / positions.len() as f32 > 0.3
    }

    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let status_text = if let Some(ref result) = self.ocr_result {
            if self.selected_chars.is_empty() {
                format!("Detected {} words - Drag to select characters", result.text_blocks.len())
            } else {
                format!("Selected {} characters", self.selected_chars.len())
            }
        } else {
            "Processing OCR...".to_string()
        };

        let title = text(status_text)
            .size(20)
            .width(Length::Fill)
            .align_x(Alignment::Center);

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

        if !self.selected_chars.is_empty() {
            let copy_btn = button(text("📋 Copy"))
                .padding([12, 24])
                .style(|theme, status| {
                    crate::app_theme::purple_button_style(theme, status)
                })
                .on_press(InteractiveOcrMessage::CopySelected);

            button_row = button_row.push(copy_btn);
        }

        let (search_button_text, is_searching) = match &self.search_state {
            SearchState::Idle => ("🔍 Search image with Google", false),
            SearchState::UploadingImage => ("📤 Uploading image...", true),
            SearchState::Completed => ("✅ Search completed", true),
            SearchState::Failed(err) => {
                log::debug!("[INTERACTIVE_OCR] Search failed with: {}", err);
                ("❌ Search failed", true)
            },
        };

        let mut search_btn = button(text(search_button_text))
            .padding([12, 24])
            .style(|theme, status| {
                crate::app_theme::primary_button_style(theme, status)
            });

        if !is_searching {
            search_btn = search_btn.on_press(InteractiveOcrMessage::SearchSelected);
        }

        let close_btn = button(text("✖ Close (Esc)"))
            .padding([12, 24])
            .style(|theme, status| {
                crate::app_theme::danger_button_style(theme, status)
            })
            .on_press(InteractiveOcrMessage::Close);

        button_row = button_row.push(search_btn).push(close_btn);

        let buttons = container(button_row)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        let mut content_column = column![title, image_with_overlay, buttons]
            .spacing(12)
            .padding(15)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center);

        if self.show_toast {
            let toast = container(
                text("✓ Text copied to clipboard")
                    .size(16)
                    .style(|_theme| {
                        iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }
                    })
            )
            .padding(12)
            .style(|_theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.098, 0.529, 0.329))),
                    text_color: Some(Color::WHITE),
                    border: iced::Border {
                        color: Color::from_rgb(0.122, 0.655, 0.408),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 8.0,
                    },
                    snap: false,
                }
            });

            content_column = content_column.push(container(toast).width(Length::Fill).align_x(Alignment::Center));
        }

        let content = content_column;

        let theme = crate::app_theme::get_theme(&self.theme_mode);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(palette.background)),
                    text_color: Some(palette.text),
                    ..Default::default()
                }
            })
            .into()
    }

    fn render_image_with_overlay(&self, _ocr_result: &OcrResult) -> Element<'_, InteractiveOcrMessage> {
        let image_viewer = image::viewer(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let ocr_overlay = OcrOverlay {
            char_positions: self.char_positions.clone(),
            image_width: self.image_width,
            image_height: self.image_height,
            selected_indices: self.selected_chars.clone(),
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
    char_positions: Vec<CharPosition>,
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

        for (idx, char_pos) in self.char_positions.iter().enumerate() {
            let rect_bounds = &char_pos.bounds;

            let scaled_x = offset_x + (rect_bounds.x * scale_x);
            let scaled_y = offset_y + (rect_bounds.y * scale_y);
            let scaled_width = rect_bounds.width * scale_x;
            let scaled_height = rect_bounds.height * scale_y;

            let is_selected = self.selected_indices.contains(&idx);

            let (fill_color, stroke_width) = if is_selected {
                (Color::from_rgba(0.3, 0.8, 0.3, 0.4), 1.5)
            } else {
                (Color::from_rgba(0.2, 0.6, 1.0, 0.15), 0.5)
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

            if is_selected {
                frame.stroke(
                    &rect_path,
                    canvas::Stroke::default()
                        .with_color(Color::from_rgb(0.2, 0.9, 0.2))
                        .with_width(stroke_width)
                );
            }
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

        match event {
            iced::Event::Keyboard(keyboard_event) => match keyboard_event {
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                } => {
                    return Some(canvas::Action::publish(InteractiveOcrMessage::Close));
                }
                _ => {}
            },
            iced::Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        for (idx, char_pos) in self.char_positions.iter().enumerate() {
                            let rect_bounds = &char_pos.bounds;
                            let scaled_x = offset_x + (rect_bounds.x * scale_x);
                            let scaled_y = offset_y + (rect_bounds.y * scale_y);
                            let scaled_width = rect_bounds.width * scale_x;
                            let scaled_height = rect_bounds.height * scale_y;

                            let char_rect = Rectangle::new(
                                Point::new(scaled_x, scaled_y),
                                Size::new(scaled_width, scaled_height)
                            );

                            if char_rect.contains(cursor_position) {
                                log::debug!("[OCR_OVERLAY] Started drag at char {}: '{}'", idx, char_pos.character);
                                return Some(canvas::Action::publish(InteractiveOcrMessage::StartDrag(idx)));
                            }
                        }
                    }
                }
                iced::mouse::Event::CursorMoved { .. } => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        for (idx, char_pos) in self.char_positions.iter().enumerate() {
                            let rect_bounds = &char_pos.bounds;
                            let scaled_x = offset_x + (rect_bounds.x * scale_x);
                            let scaled_y = offset_y + (rect_bounds.y * scale_y);
                            let scaled_width = rect_bounds.width * scale_x;
                            let scaled_height = rect_bounds.height * scale_y;

                            let char_rect = Rectangle::new(
                                Point::new(scaled_x, scaled_y),
                                Size::new(scaled_width, scaled_height)
                            );

                            if char_rect.contains(cursor_position) {
                                return Some(canvas::Action::publish(InteractiveOcrMessage::UpdateDrag(idx)));
                            }
                        }
                    }
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    return Some(canvas::Action::publish(InteractiveOcrMessage::EndDrag));
                }
                _ => {}
            },
            _ => {}
        }

        None
    }
}
