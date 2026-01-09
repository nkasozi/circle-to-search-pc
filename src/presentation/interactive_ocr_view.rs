use iced::widget::{button, canvas, container, image, row, stack, text, text_input, tooltip};
use iced::{Alignment, Border, Color, Element, Length, Point, Rectangle, Shadow, Size, Vector};

use crate::core::models::{CaptureBuffer, OcrResult, ThemeMode};
use crate::infrastructure::utils::copy_text_to_clipboard;

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Idle,
    UploadingImage,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CopyState {
    Idle,
    Success,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveState {
    Idle,
    Success(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharPosition {
    pub word_index: usize,
    pub char_index: usize,
    pub bounds: Rectangle,
    pub character: char,
}

#[derive(Debug, Clone)]
pub struct DrawStroke {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
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
    search_query: String,
    spinner_frame: usize,
    theme_mode: ThemeMode,
    copy_state: CopyState,
    save_state: SaveState,
    draw_strokes: Vec<DrawStroke>,
    current_stroke_points: Vec<Point>,
    is_drawing: bool,
    draw_color: Color,
    draw_width: f32,
    draw_mode_enabled: bool,
    show_help_hint: bool,
}
#[derive(Debug, Clone)]
pub enum InteractiveOcrMessage {
    Close,
    StartDrag(usize),
    UpdateDrag(usize),
    EndDrag,
    CopySelected,
    SearchSelected,
    SearchQueryChanged(String),
    SearchUploading,
    SearchCompleted,
    SearchFailed(String),
    SpinnerTick,
    HideToast,
    SelectAll,
    DeselectAll,
    DismissHelpHint,
    StartDrawing(Point),
    UpdateDrawing(Point),
    EndDrawing,
    CopyImageToClipboard,
    SaveImageToFile,
    Recrop,
    ToggleDrawMode,
    SetDrawColor(Color),
    ClearDrawings,
    SaveSuccess(String),
    SaveFailed(String),
    HideSaveToast,
}

impl InteractiveOcrView {
    pub fn build(capture_buffer: CaptureBuffer, theme_mode: ThemeMode) -> Self {
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
            search_query: String::new(),
            spinner_frame: 0,
            theme_mode,
            copy_state: CopyState::Idle,
            save_state: SaveState::Idle,
            draw_strokes: Vec::new(),
            current_stroke_points: Vec::new(),
            is_drawing: false,
            draw_color: Color::from_rgb(1.0, 0.0, 0.0),
            draw_width: 3.0,
            draw_mode_enabled: false,
            show_help_hint: false,
        }
    }

    pub fn get_capture_buffer(&self) -> &CaptureBuffer {
        &self.capture_buffer
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    pub fn is_searching(&self) -> bool {
        matches!(self.search_state, SearchState::UploadingImage)
    }

    pub fn get_draw_strokes(&self) -> Vec<DrawStroke> {
        self.draw_strokes.clone()
    }

    pub fn set_draw_strokes(&mut self, strokes: Vec<DrawStroke>) {
        self.draw_strokes = strokes;
    }

    pub fn set_ocr_result(&mut self, result: OcrResult) {
        log::info!(
            "[INTERACTIVE_OCR] Setting OCR result with {} text blocks",
            result.text_blocks.len()
        );

        self.char_positions = Self::calculate_char_positions(&result);
        log::info!(
            "[INTERACTIVE_OCR] Calculated {} character positions",
            self.char_positions.len()
        );
        self.ocr_result = Some(result);

        if !self.char_positions.is_empty() {
            self.show_help_hint = true;
        }
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
            InteractiveOcrMessage::Close => {}
            InteractiveOcrMessage::StartDrag(char_idx) => {
                if !self.is_selecting {
                    log::debug!(
                        "[INTERACTIVE_OCR] Starting new selection at char {}",
                        char_idx
                    );
                    self.drag_start = Some(char_idx);
                    self.is_selecting = true;
                    self.show_help_hint = false;
                } else {
                    log::debug!(
                        "[INTERACTIVE_OCR] Ending current drag session, keeping selections"
                    );
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
                log::debug!(
                    "[INTERACTIVE_OCR] Drag ended with {} chars selected",
                    self.selected_chars.len()
                );
            }
            InteractiveOcrMessage::CopySelected => {
                let selected_text = self.get_selected_text_with_layout();
                if !selected_text.is_empty() {
                    log::info!("[INTERACTIVE_OCR] Copying text: {}", selected_text);
                    match copy_text_to_clipboard(&selected_text) {
                        Ok(()) => {
                            log::info!("[INTERACTIVE_OCR] Text copied to clipboard");
                            self.copy_state = CopyState::Success;
                        }
                        Err(error) => {
                            log::error!("[INTERACTIVE_OCR] Failed to copy to clipboard: {}", error);
                            self.copy_state = CopyState::Failed;
                        }
                    }
                }
            }
            InteractiveOcrMessage::SearchSelected => {
                if matches!(self.search_state, SearchState::Idle) {
                    log::info!(
                        "[INTERACTIVE_OCR] Starting reverse image search with query: '{}'",
                        self.search_query
                    );
                    self.search_state = SearchState::UploadingImage;
                }
            }
            InteractiveOcrMessage::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            InteractiveOcrMessage::SearchUploading => {
                log::debug!("[INTERACTIVE_OCR] Search state: Uploading image");
                self.search_state = SearchState::UploadingImage;
                self.spinner_frame = 0;
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
            InteractiveOcrMessage::SpinnerTick => {
                if matches!(self.search_state, SearchState::UploadingImage) {
                    self.spinner_frame = (self.spinner_frame + 1) % 8;
                }
            }
            InteractiveOcrMessage::HideToast => {
                self.copy_state = CopyState::Idle;
            }
            InteractiveOcrMessage::SelectAll => {
                log::info!(
                    "[INTERACTIVE_OCR] Selecting all {} characters",
                    self.char_positions.len()
                );
                self.selected_chars = (0..self.char_positions.len()).collect();
                self.show_help_hint = false;
            }
            InteractiveOcrMessage::DeselectAll => {
                log::info!("[INTERACTIVE_OCR] Deselecting all characters");
                self.selected_chars.clear();
                self.is_selecting = false;
                self.drag_start = None;
            }
            InteractiveOcrMessage::DismissHelpHint => {
                self.show_help_hint = false;
            }
            InteractiveOcrMessage::StartDrawing(point) => {
                self.current_stroke_points.clear();
                self.current_stroke_points.push(point);
                self.is_drawing = true;
            }
            InteractiveOcrMessage::UpdateDrawing(point) => {
                if self.is_drawing {
                    self.current_stroke_points.push(point);
                }
            }
            InteractiveOcrMessage::EndDrawing => {
                if self.is_drawing && !self.current_stroke_points.is_empty() {
                    self.draw_strokes.push(DrawStroke {
                        points: self.current_stroke_points.clone(),
                        color: self.draw_color,
                        width: self.draw_width,
                    });
                    self.current_stroke_points.clear();
                    self.is_drawing = false;
                }
            }
            InteractiveOcrMessage::CopyImageToClipboard
            | InteractiveOcrMessage::SaveImageToFile
            | InteractiveOcrMessage::Recrop => {}
            InteractiveOcrMessage::ToggleDrawMode => {
                self.draw_mode_enabled = !self.draw_mode_enabled;
                log::info!(
                    "[INTERACTIVE_OCR] Draw mode {}",
                    if self.draw_mode_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            InteractiveOcrMessage::SetDrawColor(color) => {
                self.draw_color = color;
                log::debug!("[INTERACTIVE_OCR] Draw color changed");
            }
            InteractiveOcrMessage::ClearDrawings => {
                self.draw_strokes.clear();
                log::info!("[INTERACTIVE_OCR] Cleared all drawings");
            }
            InteractiveOcrMessage::SaveSuccess(path) => {
                self.save_state = SaveState::Success(path);
            }
            InteractiveOcrMessage::SaveFailed(error) => {
                self.save_state = SaveState::Failed(error);
            }
            InteractiveOcrMessage::HideSaveToast => {
                self.save_state = SaveState::Idle;
            }
        }
    }

    fn get_selected_text_with_layout(&self) -> String {
        if self.selected_chars.is_empty() {
            return String::new();
        }

        let mut selected_positions: Vec<&CharPosition> = self
            .selected_chars
            .iter()
            .filter_map(|&idx| self.char_positions.get(idx))
            .collect();

        if selected_positions.is_empty() {
            return String::new();
        }

        selected_positions.sort_by(|a, b| {
            let y_diff = (a.bounds.y - b.bounds.y).abs();
            let line_height_threshold = a.bounds.height * 0.5;
            if y_diff > line_height_threshold {
                a.bounds.y.partial_cmp(&b.bounds.y).unwrap()
            } else {
                a.bounds.x.partial_cmp(&b.bounds.x).unwrap()
            }
        });

        let mut result = String::new();
        let mut last_y = selected_positions[0].bounds.y;
        let mut last_word_idx = selected_positions[0].word_index;
        let mut last_x_end = selected_positions[0].bounds.x + selected_positions[0].bounds.width;

        for pos in selected_positions {
            let line_height_threshold = pos.bounds.height * 0.5;
            let y_diff = (pos.bounds.y - last_y).abs();

            if y_diff > line_height_threshold {
                result.push('\n');
                last_y = pos.bounds.y;
                last_word_idx = pos.word_index;
                last_x_end = pos.bounds.x + pos.bounds.width;
            } else if pos.word_index != last_word_idx {
                let gap_between_words = pos.bounds.x - last_x_end;
                let space_threshold = pos.bounds.width * 0.3;
                if gap_between_words > space_threshold {
                    result.push(' ');
                }
                last_word_idx = pos.word_index;
                last_x_end = pos.bounds.x + pos.bounds.width;
            } else {
                last_x_end = pos.bounds.x + pos.bounds.width;
            }
            result.push(pos.character);
        }

        result
    }

    #[allow(dead_code)]
    fn detect_vertical_layout(&self, positions: &[&CharPosition]) -> bool {
        if positions.len() < 2 {
            return false;
        }

        let mut y_changes = 0;
        for i in 1..positions.len() {
            if (positions[i].bounds.y - positions[i - 1].bounds.y).abs() > 10.0 {
                y_changes += 1;
            }
        }

        y_changes as f32 / positions.len() as f32 > 0.3
    }

    pub fn render_ui(&self) -> Element<'_, InteractiveOcrMessage> {
        let image_with_overlay = if let Some(ref ocr_result) = self.ocr_result {
            self.render_image_with_overlay(ocr_result)
        } else {
            image::viewer(self.image_handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let image_layer = container(image_with_overlay)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut layers: Vec<Element<'_, InteractiveOcrMessage>> = vec![image_layer.into()];

        let status_text = if self.draw_mode_enabled {
            "🖊️ Draw Mode ON - Click and drag to draw".to_string()
        } else if let Some(ref result) = self.ocr_result {
            if self.selected_chars.is_empty() {
                format!(
                    "Detected {} words - Click to select text",
                    result.text_blocks.len()
                )
            } else {
                format!("Selected {} characters", self.selected_chars.len())
            }
        } else {
            "Processing OCR...".to_string()
        };

        let status_banner =
            container(
                text(status_text)
                    .size(14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    }),
            )
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
            });

        let status_positioned = container(status_banner)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 16.0,
                right: 16.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(Alignment::End);

        layers.push(status_positioned.into());

        let notification_element: Option<Element<'_, InteractiveOcrMessage>> =
            match &self.copy_state {
                CopyState::Success => {
                    Some(self.build_toast("✓ Text copied!", Color::from_rgb(0.2, 0.8, 0.4)))
                }
                CopyState::Failed => {
                    Some(self.build_toast("✗ Copy failed", Color::from_rgb(0.9, 0.3, 0.3)))
                }
                CopyState::Idle => None,
            };

        if let Some(toast) = notification_element {
            let toast_positioned = container(toast)
                .width(Length::Fill)
                .padding(iced::Padding {
                    top: 60.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .align_x(Alignment::Center);
            layers.push(toast_positioned.into());
        }

        match &self.save_state {
            SaveState::Success(path) => {
                let message = format!("✓ Saved to {}", path);
                let toast = Self::build_save_toast(message, Color::from_rgb(0.2, 0.8, 0.4));
                let toast_positioned = container(toast)
                    .width(Length::Fill)
                    .padding(iced::Padding {
                        top: 100.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .align_x(Alignment::Center);
                layers.push(toast_positioned.into());
            }
            SaveState::Failed(err) => {
                let message = format!("✗ Save failed: {}", err);
                let toast = Self::build_save_toast(message, Color::from_rgb(0.9, 0.3, 0.3));
                let toast_positioned = container(toast)
                    .width(Length::Fill)
                    .padding(iced::Padding {
                        top: 100.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .align_x(Alignment::Center);
                layers.push(toast_positioned.into());
            }
            SaveState::Idle => {}
        };

        if self.show_help_hint && !self.char_positions.is_empty() {
            let help_hint = self.build_help_hint();
            let hint_positioned = container(help_hint)
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

        let mut draw_row = row![].spacing(6).align_y(Alignment::Center);

        let draw_toggle_text = if self.draw_mode_enabled {
            "🖊️"
        } else {
            "🖊️"
        };
        let draw_toggle = button(text(draw_toggle_text).size(16))
            .padding([8, 12])
            .style(move |_theme: &iced::Theme, status| {
                self.floating_btn_style(status, self.draw_mode_enabled)
            })
            .on_press(InteractiveOcrMessage::ToggleDrawMode);
        let draw_tooltip_text = if self.draw_mode_enabled {
            "Disable Draw Mode"
        } else {
            "Enable Draw Mode"
        };
        draw_row = draw_row.push(
            tooltip(draw_toggle, draw_tooltip_text, tooltip::Position::Bottom)
                .style(Self::tooltip_style),
        );

        if self.draw_mode_enabled {
            let colors = [
                (Color::from_rgb(1.0, 0.2, 0.2), ""),
                (Color::from_rgb(0.2, 0.6, 1.0), ""),
                (Color::from_rgb(0.2, 0.8, 0.2), ""),
                (Color::from_rgb(1.0, 0.85, 0.0), ""),
            ];

            for (color, _) in colors {
                let is_selected = (self.draw_color.r - color.r).abs() < 0.1
                    && (self.draw_color.g - color.g).abs() < 0.1
                    && (self.draw_color.b - color.b).abs() < 0.1;
                let color_btn = button(text("●").size(18).style(move |_theme: &iced::Theme| {
                    iced::widget::text::Style { color: Some(color) }
                }))
                .padding([6, 10])
                .style(move |_theme: &iced::Theme, status| {
                    self.color_btn_style(status, is_selected)
                })
                .on_press(InteractiveOcrMessage::SetDrawColor(color));
                draw_row = draw_row.push(color_btn);
            }

            let clear_btn = button(text("🗑").size(14))
                .padding([8, 10])
                .style(|_theme: &iced::Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => Color::from_rgba(0.8, 0.2, 0.2, 0.9),
                        button::Status::Pressed => Color::from_rgba(0.6, 0.1, 0.1, 0.9),
                        _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: Color::WHITE,
                        border: Border {
                            color: Color::from_rgba(0.5, 0.5, 0.5, 0.4),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    }
                })
                .on_press(InteractiveOcrMessage::ClearDrawings);
            draw_row = draw_row.push(
                tooltip(clear_btn, "Clear Drawings", tooltip::Position::Bottom)
                    .style(Self::tooltip_style),
            );
        }

        let draw_toolbar =
            container(draw_row)
                .padding([6, 10])
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.1, 0.1, 0.1, 0.85,
                    ))),
                    border: Border {
                        color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
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
                });

        let draw_toolbar_positioned = container(draw_toolbar)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 16.0,
                right: 0.0,
                bottom: 0.0,
                left: 16.0,
            })
            .align_x(Alignment::Start);

        layers.push(draw_toolbar_positioned.into());

        let mut action_row = row![].spacing(6).align_y(Alignment::Center);

        if !self.selected_chars.is_empty() {
            let copy_text_btn = button(text("📋 Copy Text").size(13))
                .padding([8, 14])
                .style(|_theme: &iced::Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => Color::from_rgba(0.5, 0.3, 0.8, 0.95),
                        button::Status::Pressed => Color::from_rgba(0.4, 0.2, 0.7, 0.95),
                        _ => Color::from_rgba(0.4, 0.2, 0.6, 0.9),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: Color::WHITE,
                        border: Border {
                            color: Color::from_rgba(0.6, 0.4, 0.9, 0.6),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    }
                })
                .on_press(InteractiveOcrMessage::CopySelected);
            action_row = action_row.push(
                tooltip(copy_text_btn, "Copy Selected Text", tooltip::Position::Top)
                    .style(Self::tooltip_style),
            );
        }

        let (search_text, is_searching) = match &self.search_state {
            SearchState::Idle => ("🔍", false),
            SearchState::UploadingImage => {
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                (spinner_chars[self.spinner_frame], true)
            }
            SearchState::Completed => ("✅", true),
            SearchState::Failed(_) => ("❌", true),
        };

        let search_input = text_input("Add search query...", &self.search_query)
            .on_input(InteractiveOcrMessage::SearchQueryChanged)
            .padding([6, 10])
            .width(Length::Fixed(150.0))
            .style(|_theme: &iced::Theme, _status| text_input::Style {
                background: iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.9)),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.6),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                icon: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.7),
                value: Color::WHITE,
                selection: Color::from_rgba(0.3, 0.5, 0.8, 0.5),
            });

        action_row = action_row.push(
            tooltip(
                search_input,
                "Optional: Add text to refine your search",
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );

        let mut search_btn = button(text(search_text).size(14)).padding([8, 12]).style(
            |_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.2, 0.5, 0.9, 0.95),
                    button::Status::Pressed => Color::from_rgba(0.1, 0.4, 0.8, 0.95),
                    _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.3, 0.6, 1.0, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            },
        );
        if !is_searching {
            search_btn = search_btn.on_press(InteractiveOcrMessage::SearchSelected);
        }
        action_row = action_row.push(
            tooltip(search_btn, "Search Image on Google", tooltip::Position::Top)
                .style(Self::tooltip_style),
        );

        let copy_img_btn = button(text("📷").size(14))
            .padding([8, 12])
            .style(|_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.3, 0.3, 0.3, 0.95),
                    button::Status::Pressed => Color::from_rgba(0.2, 0.2, 0.2, 0.95),
                    _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.5, 0.5, 0.5, 0.4),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .on_press(InteractiveOcrMessage::CopyImageToClipboard);
        action_row = action_row.push(
            tooltip(
                copy_img_btn,
                "Copy Image to Clipboard",
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );

        let save_btn = button(text("💾").size(14))
            .padding([8, 12])
            .style(|_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.2, 0.6, 0.3, 0.95),
                    button::Status::Pressed => Color::from_rgba(0.1, 0.5, 0.2, 0.95),
                    _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.3, 0.7, 0.4, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .on_press(InteractiveOcrMessage::SaveImageToFile);
        action_row = action_row.push(
            tooltip(save_btn, "Save Image to File", tooltip::Position::Top)
                .style(Self::tooltip_style),
        );

        let recrop_btn = button(text("🔄").size(14))
            .padding([8, 12])
            .style(|_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.4, 0.4, 0.5, 0.95),
                    button::Status::Pressed => Color::from_rgba(0.3, 0.3, 0.4, 0.95),
                    _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.5, 0.5, 0.6, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .on_press(InteractiveOcrMessage::Recrop);
        action_row = action_row.push(
            tooltip(recrop_btn, "Recrop Selection", tooltip::Position::Top)
                .style(Self::tooltip_style),
        );

        let close_btn = button(text("✖").size(14))
            .padding([8, 12])
            .style(|_theme: &iced::Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(0.8, 0.2, 0.2, 0.95),
                    button::Status::Pressed => Color::from_rgba(0.6, 0.1, 0.1, 0.95),
                    _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgba(0.7, 0.3, 0.3, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .on_press(InteractiveOcrMessage::Close);
        action_row = action_row
            .push(tooltip(close_btn, "Close", tooltip::Position::Top).style(Self::tooltip_style));

        let action_toolbar =
            container(action_row)
                .padding([6, 10])
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.1, 0.1, 0.1, 0.85,
                    ))),
                    border: Border {
                        color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
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
                });

        let action_toolbar_positioned = container(action_toolbar)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: 0.0,
                right: 0.0,
                bottom: 16.0,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .align_y(Alignment::End);

        layers.push(action_toolbar_positioned.into());

        container(stack(layers))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08))),
                ..Default::default()
            })
            .into()
    }

    fn build_toast<'a>(
        &self,
        message: &'a str,
        color: Color,
    ) -> Element<'a, InteractiveOcrMessage> {
        container(
            text(message)
                .size(14)
                .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(color),
                }),
        )
        .padding([8, 16])
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.1, 0.1, 0.1, 0.9,
            ))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            text_color: None,
            snap: false,
        })
        .into()
    }

    fn build_save_toast(message: String, color: Color) -> Element<'static, InteractiveOcrMessage> {
        container(
            text(message)
                .size(14)
                .style(move |_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(color),
                }),
        )
        .padding([8, 16])
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.1, 0.1, 0.1, 0.9,
            ))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            text_color: None,
            snap: false,
        })
        .into()
    }

    fn build_help_hint(&self) -> Element<'_, InteractiveOcrMessage> {
        let hint_content = row![
            text("💡 Click and drag on text to select • ⌘A to select all • Esc to deselect")
                .size(13)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.9, 0.9, 0.9, 0.95)),
                }),
            button(text("✕").size(12))
                .padding([4, 8])
                .style(|_theme: &iced::Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => Color::from_rgba(0.4, 0.4, 0.4, 0.8),
                        button::Status::Pressed => Color::from_rgba(0.3, 0.3, 0.3, 0.8),
                        _ => Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: Color::from_rgba(0.8, 0.8, 0.8, 0.9),
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: false,
                    }
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

    fn floating_btn_style(&self, status: button::Status, is_active: bool) -> button::Style {
        let base_bg = if is_active {
            Color::from_rgba(0.3, 0.6, 1.0, 0.9)
        } else {
            Color::from_rgba(0.15, 0.15, 0.15, 0.85)
        };
        let bg = match status {
            button::Status::Hovered => {
                if is_active {
                    Color::from_rgba(0.4, 0.7, 1.0, 0.95)
                } else {
                    Color::from_rgba(0.25, 0.25, 0.25, 0.9)
                }
            }
            button::Status::Pressed => {
                if is_active {
                    Color::from_rgba(0.2, 0.5, 0.9, 0.95)
                } else {
                    Color::from_rgba(0.2, 0.2, 0.2, 0.9)
                }
            }
            _ => base_bg,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                color: if is_active {
                    Color::from_rgba(0.4, 0.7, 1.0, 0.7)
                } else {
                    Color::from_rgba(0.5, 0.5, 0.5, 0.4)
                },
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    fn color_btn_style(&self, status: button::Status, is_selected: bool) -> button::Style {
        let bg = match status {
            button::Status::Hovered => Color::from_rgba(0.3, 0.3, 0.3, 0.9),
            button::Status::Pressed => Color::from_rgba(0.2, 0.2, 0.2, 0.9),
            _ => Color::from_rgba(0.15, 0.15, 0.15, 0.85),
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                color: if is_selected {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.8)
                } else {
                    Color::from_rgba(0.4, 0.4, 0.4, 0.4)
                },
                width: if is_selected { 2.0 } else { 1.0 },
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    fn tooltip_style(_theme: &iced::Theme) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.1, 0.1, 0.1, 0.95,
            ))),
            border: Border {
                color: Color::from_rgba(0.4, 0.4, 0.4, 0.6),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            text_color: Some(Color::WHITE),
            snap: false,
        }
    }

    fn render_image_with_overlay(
        &self,
        _ocr_result: &OcrResult,
    ) -> Element<'_, InteractiveOcrMessage> {
        let image_viewer = image::viewer(self.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let ocr_overlay = OcrOverlay {
            char_positions: self.char_positions.clone(),
            image_width: self.image_width,
            image_height: self.image_height,
            selected_indices: self.selected_chars.clone(),
            draw_strokes: self.draw_strokes.clone(),
            current_stroke_points: self.current_stroke_points.clone(),
            is_drawing: self.is_drawing,
            draw_color: self.draw_color,
            draw_width: self.draw_width,
            draw_mode_enabled: self.draw_mode_enabled,
        };

        let overlay_canvas =
            container(canvas(ocr_overlay).width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill);

        stack![image_viewer, overlay_canvas]
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
    draw_strokes: Vec<DrawStroke>,
    current_stroke_points: Vec<Point>,
    is_drawing: bool,
    draw_color: Color,
    draw_width: f32,
    draw_mode_enabled: bool,
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
                Size::new(scaled_width, scaled_height),
            );

            frame.fill_rectangle(
                Point::new(scaled_x, scaled_y),
                Size::new(scaled_width, scaled_height),
                fill_color,
            );

            if is_selected {
                frame.stroke(
                    &rect_path,
                    canvas::Stroke::default()
                        .with_color(Color::from_rgb(0.2, 0.9, 0.2))
                        .with_width(stroke_width),
                );
            }
        }

        for stroke in &self.draw_strokes {
            if stroke.points.len() > 1 {
                let mut path_builder = canvas::path::Builder::new();
                let first_point = stroke.points[0];
                let first_scaled_x = offset_x + (first_point.x / img_width) * display_width;
                let first_scaled_y = offset_y + (first_point.y / img_height) * display_height;
                path_builder.move_to(Point::new(first_scaled_x, first_scaled_y));

                for point in stroke.points.iter().skip(1) {
                    let scaled_x = offset_x + (point.x / img_width) * display_width;
                    let scaled_y = offset_y + (point.y / img_height) * display_height;
                    path_builder.line_to(Point::new(scaled_x, scaled_y));
                }

                let path = path_builder.build();
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(stroke.color)
                        .with_width(stroke.width),
                );
            }
        }

        if self.is_drawing && self.current_stroke_points.len() > 1 {
            let mut path_builder = canvas::path::Builder::new();
            let first_point = self.current_stroke_points[0];
            let first_scaled_x = offset_x + (first_point.x / img_width) * display_width;
            let first_scaled_y = offset_y + (first_point.y / img_height) * display_height;
            path_builder.move_to(Point::new(first_scaled_x, first_scaled_y));

            for point in self.current_stroke_points.iter().skip(1) {
                let scaled_x = offset_x + (point.x / img_width) * display_width;
                let scaled_y = offset_y + (point.y / img_height) * display_height;
                path_builder.line_to(Point::new(scaled_x, scaled_y));
            }

            let path = path_builder.build();
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(self.draw_color)
                    .with_width(self.draw_width),
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
                    if !self.selected_indices.is_empty() {
                        return Some(canvas::Action::publish(InteractiveOcrMessage::DeselectAll));
                    }
                    return Some(canvas::Action::publish(InteractiveOcrMessage::Close));
                }
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Character(c),
                    modifiers,
                    ..
                } => {
                    if (modifiers.command() || modifiers.control()) && c.as_str() == "a" {
                        log::debug!("[INTERACTIVE_OCR] Select all triggered via keyboard shortcut");
                        return Some(canvas::Action::publish(InteractiveOcrMessage::SelectAll));
                    }
                }
                _ => {}
            },
            iced::Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        if self.draw_mode_enabled {
                            let rel_x = (cursor_position.x - offset_x) / scale_x;
                            let rel_y = (cursor_position.y - offset_y) / scale_y;
                            return Some(canvas::Action::publish(
                                InteractiveOcrMessage::StartDrawing(Point::new(rel_x, rel_y)),
                            ));
                        }

                        for (idx, char_pos) in self.char_positions.iter().enumerate() {
                            let rect_bounds = &char_pos.bounds;
                            let scaled_x = offset_x + (rect_bounds.x * scale_x);
                            let scaled_y = offset_y + (rect_bounds.y * scale_y);
                            let scaled_width = rect_bounds.width * scale_x;
                            let scaled_height = rect_bounds.height * scale_y;

                            let char_rect = Rectangle::new(
                                Point::new(scaled_x, scaled_y),
                                Size::new(scaled_width, scaled_height),
                            );

                            if char_rect.contains(cursor_position) {
                                log::debug!(
                                    "[OCR_OVERLAY] Started drag at char {}: '{}'",
                                    idx,
                                    char_pos.character
                                );
                                return Some(canvas::Action::publish(
                                    InteractiveOcrMessage::StartDrag(idx),
                                ));
                            }
                        }
                    }
                }
                iced::mouse::Event::CursorMoved { .. } => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        if self.is_drawing {
                            let rel_x = (cursor_position.x - offset_x) / scale_x;
                            let rel_y = (cursor_position.y - offset_y) / scale_y;
                            return Some(canvas::Action::publish(
                                InteractiveOcrMessage::UpdateDrawing(Point::new(rel_x, rel_y)),
                            ));
                        }

                        for (idx, char_pos) in self.char_positions.iter().enumerate() {
                            let rect_bounds = &char_pos.bounds;
                            let scaled_x = offset_x + (rect_bounds.x * scale_x);
                            let scaled_y = offset_y + (rect_bounds.y * scale_y);
                            let scaled_width = rect_bounds.width * scale_x;
                            let scaled_height = rect_bounds.height * scale_y;

                            let char_rect = Rectangle::new(
                                Point::new(scaled_x, scaled_y),
                                Size::new(scaled_width, scaled_height),
                            );

                            if char_rect.contains(cursor_position) {
                                return Some(canvas::Action::publish(
                                    InteractiveOcrMessage::UpdateDrag(idx),
                                ));
                            }
                        }
                    }
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    if self.is_drawing {
                        return Some(canvas::Action::publish(InteractiveOcrMessage::EndDrawing));
                    } else {
                        return Some(canvas::Action::publish(InteractiveOcrMessage::EndDrag));
                    }
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right) => {
                    return Some(canvas::Action::publish(InteractiveOcrMessage::EndDrawing));
                }
                _ => {}
            },
            _ => {}
        }

        None
    }
}
