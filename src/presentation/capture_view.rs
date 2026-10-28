use iced::{Point, Rectangle, Size};

mod canvas_program;
mod geometry;
mod selection;
mod ui;

#[cfg(test)]
mod tests;

use crate::core::models::CaptureBuffer;

const MINIMUM_SELECTION_EDGE: f32 = 8.0;
const LOG_TAG: &str = "[CAPTURE_VIEW]";
const VIEW_BUILT_EVENT: &str = "view_built";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawMode {
    Rectangle,
    Freeform,
}

pub struct CaptureView {
    capture_buffer: CaptureBuffer,
    selection_start: Option<Point>,
    selection_current: Option<Point>,
    is_selecting: bool,
    viewer_bounds: std::cell::Cell<Rectangle>,
    draw_mode: DrawMode,
    freeform_points: Vec<Point>,
    is_shape_closed: bool,
}

#[derive(Debug, Clone)]
pub enum CaptureViewMessage {
    MousePressed(Point),
    MouseMoved(Point),
    MouseReleased,
    ConfirmSelection,
    #[allow(dead_code)]
    CancelRequested,
    SetDrawMode(DrawMode),
    SelectWindow,
    SwitchToRecording,
}

impl CaptureView {
    pub fn build_with_capture_buffer(capture_buffer: CaptureBuffer) -> Self {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": VIEW_BUILT_EVENT})
        );
        Self {
            capture_buffer,
            selection_start: None,
            selection_current: None,
            is_selecting: false,
            viewer_bounds: std::cell::Cell::new(Rectangle::new(Point::ORIGIN, Size::ZERO)),
            draw_mode: DrawMode::Rectangle,
            freeform_points: Vec::new(),
            is_shape_closed: false,
        }
    }

    pub fn update(&mut self, message: CaptureViewMessage) -> bool {
        match message {
            CaptureViewMessage::MousePressed(point) => self.handle_mouse_pressed(point),
            CaptureViewMessage::MouseMoved(point) => self.handle_mouse_moved(point),
            CaptureViewMessage::MouseReleased => self.handle_mouse_released(),
            CaptureViewMessage::CancelRequested => self.reset_selection(),
            CaptureViewMessage::SetDrawMode(mode) => self.handle_set_draw_mode(mode),
            CaptureViewMessage::ConfirmSelection
            | CaptureViewMessage::SelectWindow
            | CaptureViewMessage::SwitchToRecording => false,
        }
    }

    pub fn has_completed_selection(&self) -> bool {
        if self.is_selecting {
            return false;
        }
        let is_shape_ready = match self.draw_mode {
            DrawMode::Rectangle => true,
            DrawMode::Freeform => self.freeform_points.len() > 2,
        };
        is_shape_ready
            && self
                .calculate_selection_rectangle()
                .is_some_and(|(_, size)| {
                    size.width >= MINIMUM_SELECTION_EDGE && size.height >= MINIMUM_SELECTION_EDGE
                })
    }

    pub fn get_selected_region(&self) -> Option<Rectangle> {
        self.calculate_selection_rectangle()
            .map(|(point, size)| self.map_selection_to_image(Rectangle::new(point, size)))
    }

    pub fn get_capture_buffer(&self) -> &CaptureBuffer {
        &self.capture_buffer
    }
}
