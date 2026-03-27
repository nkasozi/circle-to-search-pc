use iced::{Point, Rectangle, Size};

mod canvas_program;
mod ui;

use crate::core::models::CaptureBuffer;

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
}

impl CaptureView {
    pub fn build_with_capture_buffer(capture_buffer: CaptureBuffer) -> Self {
        log::debug!("[CAPTURE_VIEW] building view");
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

    pub fn update(&mut self, message: CaptureViewMessage) {
        match message {
            CaptureViewMessage::MousePressed(point) => match self.draw_mode {
                DrawMode::Rectangle => {
                    self.selection_start = Some(point);
                    self.selection_current = Some(point);
                    self.is_selecting = true;
                }
                DrawMode::Freeform => {
                    if !self.is_selecting {
                        self.freeform_points.clear();
                        self.freeform_points.push(point);
                        self.is_selecting = true;
                        self.is_shape_closed = false;
                    }
                }
            },
            CaptureViewMessage::MouseMoved(point) => {
                if self.is_selecting {
                    match self.draw_mode {
                        DrawMode::Rectangle => {
                            self.selection_current = Some(point);
                        }
                        DrawMode::Freeform => {
                            self.freeform_points.push(point);

                            if self.freeform_points.len() > 10 {
                                let first = self.freeform_points[0];
                                let distance = ((point.x - first.x).powi(2)
                                    + (point.y - first.y).powi(2))
                                .sqrt();
                                if distance < 20.0 {
                                    self.is_shape_closed = true;
                                    self.is_selecting = false;
                                }
                            }
                        }
                    }
                }
            }
            CaptureViewMessage::MouseReleased => {
                if self.draw_mode == DrawMode::Freeform
                    && self.is_selecting
                    && self.freeform_points.len() > 2
                {
                    let first = self.freeform_points[0];
                    let Some(last) = self.freeform_points.last().copied() else {
                        self.is_selecting = false;
                        return;
                    };
                    let distance = ((last.x - first.x).powi(2) + (last.y - first.y).powi(2)).sqrt();
                    if distance < 30.0 {
                        self.is_shape_closed = true;
                    }
                }
                self.is_selecting = false;
            }
            CaptureViewMessage::ConfirmSelection => {}
            CaptureViewMessage::CancelRequested => {
                self.selection_start = None;
                self.selection_current = None;
                self.is_selecting = false;
                self.freeform_points.clear();
                self.is_shape_closed = false;
            }
            CaptureViewMessage::SetDrawMode(mode) => {
                self.draw_mode = mode;
                self.selection_start = None;
                self.selection_current = None;
                self.is_selecting = false;
                self.freeform_points.clear();
                self.is_shape_closed = false;
            }
            CaptureViewMessage::SelectWindow => {}
        }
    }

    fn calculate_selection_rectangle(&self) -> Option<(Point, Size)> {
        match self.draw_mode {
            DrawMode::Rectangle => match (self.selection_start, self.selection_current) {
                (Some(start), Some(current)) => {
                    let x = start.x.min(current.x);
                    let y = start.y.min(current.y);
                    let width = (start.x - current.x).abs();
                    let height = (start.y - current.y).abs();
                    Some((Point::new(x, y), Size::new(width, height)))
                }
                _ => None,
            },
            DrawMode::Freeform => {
                if self.freeform_points.is_empty() {
                    return None;
                }

                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;

                for point in &self.freeform_points {
                    min_x = min_x.min(point.x);
                    min_y = min_y.min(point.y);
                    max_x = max_x.max(point.x);
                    max_y = max_y.max(point.y);
                }

                let padding = 10.0;
                let x = (min_x - padding).max(0.0);
                let y = (min_y - padding).max(0.0);
                let width = max_x - min_x + (2.0 * padding);
                let height = max_y - min_y + (2.0 * padding);

                Some((Point::new(x, y), Size::new(width, height)))
            }
        }
    }

    pub fn get_selected_region(&self) -> Option<Rectangle> {
        self.calculate_selection_rectangle().map(|(point, size)| {
            let selection_rect = Rectangle::new(point, size);

            let viewer_bounds = self.viewer_bounds.get();
            if viewer_bounds.width == 0.0 || viewer_bounds.height == 0.0 {
                log::warn!("[CAPTURE_VIEW] Viewer bounds not set, using raw selection");
                return selection_rect;
            }

            let scale_x = self.capture_buffer.width as f32 / viewer_bounds.width;
            let scale_y = self.capture_buffer.height as f32 / viewer_bounds.height;

            let image_x = (selection_rect.x - viewer_bounds.x) * scale_x;
            let image_y = (selection_rect.y - viewer_bounds.y) * scale_y;
            let image_width = selection_rect.width * scale_x;
            let image_height = selection_rect.height * scale_y;

            log::debug!(
                "[CAPTURE_VIEW] Selection coords: {:?} -> Image coords: ({}, {}) {}x{}",
                selection_rect,
                image_x,
                image_y,
                image_width,
                image_height
            );
            log::debug!(
                "[CAPTURE_VIEW] Viewer bounds: {:?}, Image size: {}x{}, Scale: ({}, {})",
                viewer_bounds,
                self.capture_buffer.width,
                self.capture_buffer.height,
                scale_x,
                scale_y
            );

            Rectangle::new(
                Point::new(image_x, image_y),
                Size::new(image_width, image_height),
            )
        })
    }

    pub fn get_capture_buffer(&self) -> &CaptureBuffer {
        &self.capture_buffer
    }
}
