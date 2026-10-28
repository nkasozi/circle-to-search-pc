use iced::keyboard::{self, key};
use iced::mouse;
use iced::{Point, Rectangle, Size};

use super::{DisplayLayout, InteractiveOcrMessage, OcrOverlay};

const SELECT_ALL_SHORTCUT_KEY: &str = "a";
const COPY_TEXT_SHORTCUT_KEY: &str = "c";
const SAVE_IMAGE_SHORTCUT_KEY: &str = "s";
const COPY_IMAGE_SHORTCUT_KEY: &str = "d";
const DRAW_PANEL_HANDLE_SIZE: Size = Size::new(300.0, 26.0);
const LOG_TAG: &str = "[OCR_OVERLAY]";
const INTERACTIVE_OCR_LOG_TAG: &str = "[INTERACTIVE_OCR]";
const DRAW_PANEL_DRAG_STARTED_EVENT: &str = "draw_panel_drag_started";
const CHARACTER_DRAG_STARTED_EVENT: &str = "character_drag_started";
const SELECT_ALL_SHORTCUT_EVENT: &str = "select_all_shortcut_triggered";
const COPY_TEXT_SHORTCUT_EVENT: &str = "copy_text_shortcut_triggered";
const SAVE_IMAGE_SHORTCUT_EVENT: &str = "save_image_shortcut_triggered";
const COPY_IMAGE_SHORTCUT_EVENT: &str = "copy_image_shortcut_triggered";

impl OcrOverlay {
    pub(super) fn handle_keyboard_event(
        &self,
        keyboard_event: &keyboard::Event,
    ) -> Option<InteractiveOcrMessage> {
        match keyboard_event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            } => Some(self.escape_message()),
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(character),
                modifiers,
                ..
            } => {
                let is_command_or_control = modifiers.command() || modifiers.control();
                is_command_or_control
                    .then(|| shortcut_message_for(character.as_str()))
                    .flatten()
            }
            _ => None,
        }
    }

    pub(super) fn handle_mouse_event(
        &self,
        mouse_event: &mouse::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<InteractiveOcrMessage> {
        let display_layout =
            DisplayLayout::fit(self.image_width as f32, self.image_height as f32, bounds);
        match mouse_event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                let cursor_position = cursor.position_in(bounds)?;
                self.handle_left_press(&display_layout, cursor_position)
            }
            mouse::Event::CursorMoved { .. } => {
                let cursor_position = cursor.position_in(bounds)?;
                self.handle_cursor_moved(&display_layout, cursor_position)
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => Some(self.left_release_message()),
            mouse::Event::ButtonReleased(mouse::Button::Right) => {
                Some(InteractiveOcrMessage::EndDrawing)
            }
            _ => None,
        }
    }

    fn escape_message(&self) -> InteractiveOcrMessage {
        match self.selected_indices.is_empty() {
            true => InteractiveOcrMessage::Close,
            false => InteractiveOcrMessage::DeselectAll,
        }
    }

    fn handle_left_press(
        &self,
        display_layout: &DisplayLayout,
        cursor_position: Point,
    ) -> Option<InteractiveOcrMessage> {
        let handle_strip = Rectangle::new(self.draw_panel_position, DRAW_PANEL_HANDLE_SIZE);
        if handle_strip.contains(cursor_position) {
            log::debug!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": DRAW_PANEL_DRAG_STARTED_EVENT, "cursor_x": cursor_position.x, "cursor_y": cursor_position.y})
            );
            return Some(InteractiveOcrMessage::DrawPanelDragStarted(
                cursor_position.x,
                cursor_position.y,
            ));
        }
        if self.draw_mode_enabled {
            return Some(InteractiveOcrMessage::StartDrawing(
                display_layout.to_image_point(cursor_position),
            ));
        }
        let character_index = self.find_character_at(display_layout, cursor_position)?;
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CHARACTER_DRAG_STARTED_EVENT, "character_index": character_index, "character": self.char_positions[character_index].character})
        );
        Some(InteractiveOcrMessage::StartDrag(character_index))
    }

    fn handle_cursor_moved(
        &self,
        display_layout: &DisplayLayout,
        cursor_position: Point,
    ) -> Option<InteractiveOcrMessage> {
        if self.draw_panel_is_dragging {
            return Some(InteractiveOcrMessage::DrawPanelMoved(
                cursor_position.x,
                cursor_position.y,
            ));
        }
        if self.is_drawing {
            return Some(InteractiveOcrMessage::UpdateDrawing(
                display_layout.to_image_point(cursor_position),
            ));
        }
        self.find_character_at(display_layout, cursor_position)
            .map(InteractiveOcrMessage::UpdateDrag)
    }

    fn left_release_message(&self) -> InteractiveOcrMessage {
        match (self.draw_panel_is_dragging, self.is_drawing) {
            (true, _) => InteractiveOcrMessage::DrawPanelReleased,
            (false, true) => InteractiveOcrMessage::EndDrawing,
            (false, false) => InteractiveOcrMessage::EndDrag,
        }
    }

    fn find_character_at(
        &self,
        display_layout: &DisplayLayout,
        cursor_position: Point,
    ) -> Option<usize> {
        self.char_positions.iter().position(|character_position| {
            display_layout
                .to_canvas_rectangle(&character_position.bounds)
                .contains(cursor_position)
        })
    }
}

fn shortcut_message_for(character: &str) -> Option<InteractiveOcrMessage> {
    match character {
        SELECT_ALL_SHORTCUT_KEY => {
            log::debug!(
                "{} {}",
                INTERACTIVE_OCR_LOG_TAG,
                serde_json::json!({"event": SELECT_ALL_SHORTCUT_EVENT})
            );
            Some(InteractiveOcrMessage::SelectAll)
        }
        COPY_TEXT_SHORTCUT_KEY => {
            log::debug!(
                "{} {}",
                INTERACTIVE_OCR_LOG_TAG,
                serde_json::json!({"event": COPY_TEXT_SHORTCUT_EVENT})
            );
            Some(InteractiveOcrMessage::CopySelected)
        }
        SAVE_IMAGE_SHORTCUT_KEY => {
            log::debug!(
                "{} {}",
                INTERACTIVE_OCR_LOG_TAG,
                serde_json::json!({"event": SAVE_IMAGE_SHORTCUT_EVENT})
            );
            Some(InteractiveOcrMessage::SaveImageToFile)
        }
        COPY_IMAGE_SHORTCUT_KEY => {
            log::debug!(
                "{} {}",
                INTERACTIVE_OCR_LOG_TAG,
                serde_json::json!({"event": COPY_IMAGE_SHORTCUT_EVENT})
            );
            Some(InteractiveOcrMessage::CopyImageToClipboard)
        }
        _ => None,
    }
}
