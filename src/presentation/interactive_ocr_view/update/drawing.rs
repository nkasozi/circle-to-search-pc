use iced::{Point, Vector};

use super::*;

const DRAW_MODE_ENABLED_LABEL: &str = "enabled";
const DRAW_MODE_DISABLED_LABEL: &str = "disabled";
const DRAW_MODE_TOGGLED_EVENT: &str = "draw_mode_toggled";
const DRAW_COLOR_CHANGED_EVENT: &str = "draw_color_changed";
const DRAWINGS_CLEARED_EVENT: &str = "drawings_cleared";
const DRAW_PANEL_DRAG_STARTED_EVENT: &str = "draw_panel_drag_started";
const DRAW_PANEL_DRAG_ENDED_EVENT: &str = "draw_panel_drag_ended";

impl InteractiveOcrView {
    pub(super) fn try_apply_drawing_message(
        &mut self,
        message: InteractiveOcrMessage,
    ) -> Option<InteractiveOcrMessage> {
        let _drawing_changed = match message {
            InteractiveOcrMessage::StartDrawing(point) => self.handle_start_drawing(point),
            InteractiveOcrMessage::UpdateDrawing(point) => self.handle_update_drawing(point),
            InteractiveOcrMessage::EndDrawing => self.handle_end_drawing(),
            InteractiveOcrMessage::ToggleDrawMode => self.handle_toggle_draw_mode(),
            InteractiveOcrMessage::SetDrawColor(color) => self.handle_set_draw_color(color),
            InteractiveOcrMessage::ClearDrawings => self.handle_clear_drawings(),
            InteractiveOcrMessage::DrawPanelDragStarted(cursor_x, cursor_y) => {
                self.handle_draw_panel_drag_started(cursor_x, cursor_y)
            }
            InteractiveOcrMessage::DrawPanelMoved(cursor_x, cursor_y) => {
                self.handle_draw_panel_moved(cursor_x, cursor_y)
            }
            InteractiveOcrMessage::DrawPanelReleased => self.handle_draw_panel_released(),
            other_message => return Some(other_message),
        };
        None
    }

    fn handle_start_drawing(&mut self, point: Point) -> bool {
        self.current_stroke_points.clear();
        self.current_stroke_points.push(point);
        self.is_drawing = true;
        true
    }

    fn handle_update_drawing(&mut self, point: Point) -> bool {
        if !self.is_drawing {
            return false;
        }

        self.current_stroke_points.push(point);
        true
    }

    fn handle_end_drawing(&mut self) -> bool {
        if !self.is_drawing || self.current_stroke_points.is_empty() {
            return false;
        }

        self.draw_strokes.push(DrawStroke {
            points: self.current_stroke_points.clone(),
            color: self.draw_color,
            width: self.draw_width,
        });
        self.current_stroke_points.clear();
        self.is_drawing = false;
        true
    }

    fn handle_toggle_draw_mode(&mut self) -> bool {
        self.draw_mode_enabled = !self.draw_mode_enabled;
        let draw_mode_label = match self.draw_mode_enabled {
            true => DRAW_MODE_ENABLED_LABEL,
            false => DRAW_MODE_DISABLED_LABEL,
        };
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DRAW_MODE_TOGGLED_EVENT, "draw_mode": draw_mode_label})
        );
        true
    }

    fn handle_set_draw_color(&mut self, color: iced::Color) -> bool {
        self.draw_color = color;
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DRAW_COLOR_CHANGED_EVENT})
        );
        true
    }

    fn handle_clear_drawings(&mut self) -> bool {
        self.draw_strokes.clear();
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DRAWINGS_CLEARED_EVENT})
        );
        true
    }

    fn handle_draw_panel_drag_started(&mut self, cursor_x: f32, cursor_y: f32) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DRAW_PANEL_DRAG_STARTED_EVENT, "cursor_x": cursor_x, "cursor_y": cursor_y})
        );
        self.draw_panel_is_dragging = true;
        self.draw_panel_drag_offset = Some(Vector {
            x: cursor_x - self.draw_panel_position.x,
            y: cursor_y - self.draw_panel_position.y,
        });
        true
    }

    fn handle_draw_panel_moved(&mut self, cursor_x: f32, cursor_y: f32) -> bool {
        if !self.draw_panel_is_dragging {
            return false;
        }

        let Some(offset) = self.draw_panel_drag_offset else {
            return false;
        };

        self.draw_panel_position = Point {
            x: (cursor_x - offset.x).max(0.0),
            y: (cursor_y - offset.y).max(0.0),
        };
        true
    }

    fn handle_draw_panel_released(&mut self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DRAW_PANEL_DRAG_ENDED_EVENT})
        );
        self.draw_panel_is_dragging = false;
        self.draw_panel_drag_offset = None;
        true
    }
}
