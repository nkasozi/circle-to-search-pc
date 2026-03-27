use iced::{Point, Vector};

use super::*;

const TOOLBAR_TOP_OFFSET: f32 = 500.0;

impl InteractiveOcrView {
    pub fn update(&mut self, message: InteractiveOcrMessage) {
        match message {
            InteractiveOcrMessage::Close => {}
            InteractiveOcrMessage::StartDrag(char_index) => self.handle_start_drag(char_index),
            InteractiveOcrMessage::UpdateDrag(char_index) => self.handle_update_drag(char_index),
            InteractiveOcrMessage::EndDrag => self.handle_end_drag(),
            InteractiveOcrMessage::CopySelected => self.handle_copy_selected(),
            InteractiveOcrMessage::SearchSelected => self.handle_search_selected(),
            InteractiveOcrMessage::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            InteractiveOcrMessage::SearchUploading => self.handle_search_uploading(),
            InteractiveOcrMessage::SearchCompleted => self.handle_search_completed(),
            InteractiveOcrMessage::SearchFailed(error) => self.handle_search_failed(error),
            InteractiveOcrMessage::SpinnerTick => self.handle_spinner_tick(),
            InteractiveOcrMessage::HideToast => self.handle_hide_toast(),
            InteractiveOcrMessage::SelectAll => self.handle_select_all(),
            InteractiveOcrMessage::DeselectAll => self.handle_deselect_all(),
            InteractiveOcrMessage::DismissHelpHint => {
                self.show_help_hint = false;
            }
            InteractiveOcrMessage::StartDrawing(point) => self.handle_start_drawing(point),
            InteractiveOcrMessage::UpdateDrawing(point) => self.handle_update_drawing(point),
            InteractiveOcrMessage::EndDrawing => self.handle_end_drawing(),
            InteractiveOcrMessage::CopyImageToClipboard
            | InteractiveOcrMessage::SaveImageToFile
            | InteractiveOcrMessage::Recrop => {}
            InteractiveOcrMessage::CopyImagePreparing => self.handle_copy_image_preparing(),
            InteractiveOcrMessage::CopyImageCopying => self.handle_copy_image_copying(),
            InteractiveOcrMessage::CopyImageSuccess => self.handle_copy_image_success(),
            InteractiveOcrMessage::CopyImageFailed(error) => self.handle_copy_image_failed(error),
            InteractiveOcrMessage::SaveImagePreparing => self.handle_save_image_preparing(),
            InteractiveOcrMessage::SaveImageSaving => self.handle_save_image_saving(),
            InteractiveOcrMessage::SaveSuccess(path) => {
                self.save_state = SaveState::Success(path);
            }
            InteractiveOcrMessage::SaveFailed(error) => {
                self.save_state = SaveState::Failed(error);
            }
            InteractiveOcrMessage::HideSaveToast => {
                self.save_state = SaveState::Idle;
            }
            InteractiveOcrMessage::ToggleDrawMode => self.handle_toggle_draw_mode(),
            InteractiveOcrMessage::SetDrawColor(color) => self.handle_set_draw_color(color),
            InteractiveOcrMessage::ClearDrawings => self.handle_clear_drawings(),
            InteractiveOcrMessage::ToggleToolbarPosition => self.handle_toggle_toolbar_position(),
            InteractiveOcrMessage::StartOcr => self.handle_start_ocr(),
            InteractiveOcrMessage::CancelOcr => self.handle_cancel_ocr(),
            InteractiveOcrMessage::ClearOcrOverlay => self.handle_clear_ocr_overlay(),
            InteractiveOcrMessage::OcrFailed(error) => self.handle_ocr_failed(error),
            InteractiveOcrMessage::RetryOcr => self.handle_retry_ocr(),
            InteractiveOcrMessage::DrawPanelDragStarted(cursor_x, cursor_y) => {
                self.handle_draw_panel_drag_started(cursor_x, cursor_y)
            }
            InteractiveOcrMessage::DrawPanelMoved(cursor_x, cursor_y) => {
                self.handle_draw_panel_moved(cursor_x, cursor_y)
            }
            InteractiveOcrMessage::DrawPanelReleased => self.handle_draw_panel_released(),
        }
    }

    fn handle_start_drag(&mut self, char_index: usize) {
        if self.is_selecting {
            log::debug!("[INTERACTIVE_OCR] Ending current drag session, keeping selections");
            self.is_selecting = false;
            self.drag_start = None;
            return;
        }

        log::debug!(
            "[INTERACTIVE_OCR] Starting new selection at char {}",
            char_index
        );
        self.drag_start = Some(char_index);
        self.is_selecting = true;
        self.show_help_hint = false;
    }

    fn handle_update_drag(&mut self, char_index: usize) {
        if !self.is_selecting {
            return;
        }

        let Some(start_index) = self.drag_start else {
            return;
        };

        let min_index = start_index.min(char_index);
        let max_index = start_index.max(char_index);
        let mut combined_selection = self.selected_chars.clone();

        for selected_index in min_index..=max_index {
            if !combined_selection.contains(&selected_index) {
                combined_selection.push(selected_index);
            }
        }

        combined_selection.sort_unstable();
        self.selected_chars = combined_selection;
    }

    fn handle_end_drag(&self) {
        log::debug!(
            "[INTERACTIVE_OCR] Drag ended with {} chars selected",
            self.selected_chars.len()
        );
    }

    fn handle_copy_selected(&mut self) {
        let selected_text = self.get_selected_text_with_layout();

        if selected_text.is_empty() {
            return;
        }

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

    fn handle_search_selected(&mut self) {
        if !matches!(self.search_state, SearchState::Idle) {
            return;
        }

        log::info!(
            "[INTERACTIVE_OCR] Starting reverse image search with query: '{}'",
            self.search_query
        );
        self.search_state = SearchState::UploadingImage;
    }

    fn handle_search_uploading(&mut self) {
        log::debug!("[INTERACTIVE_OCR] Search state: Uploading image");
        self.search_state = SearchState::UploadingImage;
        self.spinner_frame = 0;
    }

    fn handle_search_completed(&mut self) {
        log::info!("[INTERACTIVE_OCR] Search completed successfully");
        self.search_state = SearchState::Completed;
        self.search_state = SearchState::Idle;
    }

    fn handle_search_failed(&mut self, error: String) {
        log::error!("[INTERACTIVE_OCR] Search failed: {}", error);
        self.search_state = SearchState::Failed(error);
        self.search_state = SearchState::Idle;
    }

    fn handle_spinner_tick(&mut self) {
        if !self.should_advance_spinner() {
            return;
        }

        self.spinner_frame = (self.spinner_frame + 1) % 8;
    }

    fn should_advance_spinner(&self) -> bool {
        matches!(self.search_state, SearchState::UploadingImage)
            || matches!(
                self.image_copy_state,
                ImageCopyState::Preparing | ImageCopyState::Copying
            )
            || matches!(self.save_state, SaveState::Preparing | SaveState::Saving)
    }

    fn handle_hide_toast(&mut self) {
        self.copy_state = CopyState::Idle;
        self.image_copy_state = ImageCopyState::Idle;

        if matches!(
            self.save_state,
            SaveState::Success(_) | SaveState::Failed(_)
        ) {
            self.save_state = SaveState::Idle;
        }
    }

    fn handle_select_all(&mut self) {
        log::info!(
            "[INTERACTIVE_OCR] Selecting all {} characters",
            self.char_positions.len()
        );
        self.selected_chars = (0..self.char_positions.len()).collect();
        self.show_help_hint = false;
    }

    fn handle_deselect_all(&mut self) {
        log::info!("[INTERACTIVE_OCR] Deselecting all characters");
        self.selected_chars.clear();
        self.is_selecting = false;
        self.drag_start = None;
    }

    fn handle_start_drawing(&mut self, point: Point) {
        self.current_stroke_points.clear();
        self.current_stroke_points.push(point);
        self.is_drawing = true;
    }

    fn handle_update_drawing(&mut self, point: Point) {
        if !self.is_drawing {
            return;
        }

        self.current_stroke_points.push(point);
    }

    fn handle_end_drawing(&mut self) {
        if !self.is_drawing || self.current_stroke_points.is_empty() {
            return;
        }

        self.draw_strokes.push(DrawStroke {
            points: self.current_stroke_points.clone(),
            color: self.draw_color,
            width: self.draw_width,
        });
        self.current_stroke_points.clear();
        self.is_drawing = false;
    }

    fn handle_copy_image_preparing(&mut self) {
        log::debug!("[INTERACTIVE_OCR] Preparing to copy image");
        self.image_copy_state = ImageCopyState::Preparing;
    }

    fn handle_copy_image_copying(&mut self) {
        log::debug!("[INTERACTIVE_OCR] Copying image to clipboard");
        self.image_copy_state = ImageCopyState::Copying;
    }

    fn handle_copy_image_success(&mut self) {
        log::info!("[INTERACTIVE_OCR] Image copied to clipboard successfully");
        self.image_copy_state = ImageCopyState::Success;
    }

    fn handle_copy_image_failed(&mut self, error: String) {
        log::error!("[INTERACTIVE_OCR] Failed to copy image: {}", error);
        self.image_copy_state = ImageCopyState::Failed(error);
    }

    fn handle_save_image_preparing(&mut self) {
        log::debug!("[INTERACTIVE_OCR] Preparing to save image");
        self.save_state = SaveState::Preparing;
    }

    fn handle_save_image_saving(&mut self) {
        log::debug!("[INTERACTIVE_OCR] Saving image to file");
        self.save_state = SaveState::Saving;
    }

    fn handle_toggle_draw_mode(&mut self) {
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

    fn handle_set_draw_color(&mut self, color: iced::Color) {
        self.draw_color = color;
        log::debug!("[INTERACTIVE_OCR] Draw color changed");
    }

    fn handle_clear_drawings(&mut self) {
        self.draw_strokes.clear();
        log::info!("[INTERACTIVE_OCR] Cleared all drawings");
    }

    fn handle_toggle_toolbar_position(&mut self) {
        if self.toolbar_offset.y > 50.0 {
            self.toolbar_offset = Vector::new(0.0, 0.0);
            log::debug!("[INTERACTIVE_OCR] Moved toolbar to bottom");
            return;
        }

        self.toolbar_offset = Vector::new(0.0, TOOLBAR_TOP_OFFSET);
        log::debug!("[INTERACTIVE_OCR] Moved toolbar to top");
    }

    fn handle_start_ocr(&mut self) {
        log::info!("[INTERACTIVE_OCR] User requested OCR start");
        self.ocr_state = OcrState::Processing;
    }

    fn handle_cancel_ocr(&self) {
        log::info!("[INTERACTIVE_OCR] OCR cancellation requested by user");
    }

    fn handle_clear_ocr_overlay(&mut self) {
        log::info!("[INTERACTIVE_OCR] User cleared OCR overlay");
        self.ocr_result = None;
        self.char_positions.clear();
        self.selected_chars.clear();
        self.ocr_state = OcrState::Idle;
    }

    fn handle_ocr_failed(&mut self, error: String) {
        log::error!("[INTERACTIVE_OCR] OCR failed: {}", error);
        self.ocr_state = OcrState::Failed(error);
    }

    fn handle_retry_ocr(&mut self) {
        log::info!("[INTERACTIVE_OCR] Retrying OCR process");
        self.ocr_state = OcrState::Idle;
        self.ocr_result = None;
        self.char_positions.clear();
        self.selected_chars.clear();
    }

    fn handle_draw_panel_drag_started(&mut self, cursor_x: f32, cursor_y: f32) {
        log::debug!(
            "[INTERACTIVE_OCR] Draw panel drag started at ({}, {})",
            cursor_x,
            cursor_y
        );
        self.draw_panel_is_dragging = true;
        self.draw_panel_drag_offset = Some(Vector {
            x: cursor_x - self.draw_panel_position.x,
            y: cursor_y - self.draw_panel_position.y,
        });
    }

    fn handle_draw_panel_moved(&mut self, cursor_x: f32, cursor_y: f32) {
        if !self.draw_panel_is_dragging {
            return;
        }

        let Some(offset) = self.draw_panel_drag_offset else {
            return;
        };

        self.draw_panel_position = Point {
            x: (cursor_x - offset.x).max(0.0),
            y: (cursor_y - offset.y).max(0.0),
        };
    }

    fn handle_draw_panel_released(&mut self) {
        log::debug!("[INTERACTIVE_OCR] Draw panel drag ended");
        self.draw_panel_is_dragging = false;
        self.draw_panel_drag_offset = None;
    }
}
