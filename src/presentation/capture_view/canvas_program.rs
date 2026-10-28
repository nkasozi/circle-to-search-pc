use super::*;
use iced::mouse;
use iced::widget::canvas;
use iced::{Color, Rectangle};

use crate::presentation::design::tokens::vibrant;

mod masks;

use masks::{fill_idle_mask, fill_mask_around_selection, fit_image_in_bounds};

const SELECTION_MASK_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
const RECTANGLE_SELECTION_STROKE_WIDTH: f32 = 2.0;
const FREEFORM_BOUNDING_BOX_ALPHA: f32 = 0.6;
const FREEFORM_BOUNDING_BOX_STROKE_WIDTH: f32 = 1.0;
const FREEFORM_PATH_STROKE_WIDTH: f32 = 3.0;

impl canvas::Program<CaptureViewMessage> for CaptureView {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<CaptureViewMessage>> {
        match event {
            iced::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => match cursor {
                    mouse::Cursor::Available(position) => Some(canvas::Action::publish(
                        CaptureViewMessage::MousePressed(position),
                    )),
                    _ => None,
                },
                mouse::Event::CursorMoved { .. } => match cursor {
                    mouse::Cursor::Available(position) => Some(canvas::Action::publish(
                        CaptureViewMessage::MouseMoved(position),
                    )),
                    _ => None,
                },
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    Some(canvas::Action::publish(CaptureViewMessage::MouseReleased))
                }
                _ => None,
            },
            iced::Event::Keyboard(keyboard_event) => match keyboard_event {
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
                    ..
                } => Some(canvas::Action::publish(
                    CaptureViewMessage::ConfirmSelection,
                )),
                _ => None,
            },
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        self.viewer_bounds.set(fit_image_in_bounds(
            self.capture_buffer.width as f32,
            self.capture_buffer.height as f32,
            bounds,
        ));
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let _overlay_drawn = match self.draw_mode {
            DrawMode::Rectangle => self.draw_rectangle_overlay(&mut frame, bounds),
            DrawMode::Freeform => self.draw_freeform_overlay(&mut frame, bounds),
        };

        vec![frame.into_geometry()]
    }
}

impl CaptureView {
    fn draw_rectangle_overlay(
        &self,
        frame: &mut canvas::Frame<iced::Renderer>,
        bounds: Rectangle,
    ) -> bool {
        let Some((top_left, size)) = self.calculate_selection_rectangle() else {
            return fill_idle_mask(frame, bounds);
        };
        let _mask_filled = fill_mask_around_selection(
            frame,
            bounds,
            Rectangle::new(top_left, size),
            SELECTION_MASK_COLOR,
        );
        frame.stroke(
            &canvas::Path::rectangle(top_left, size),
            canvas::Stroke::default()
                .with_color(vibrant::VIOLET)
                .with_width(RECTANGLE_SELECTION_STROKE_WIDTH),
        );
        true
    }

    fn draw_freeform_overlay(
        &self,
        frame: &mut canvas::Frame<iced::Renderer>,
        bounds: Rectangle,
    ) -> bool {
        if self.freeform_points.is_empty() {
            return fill_idle_mask(frame, bounds);
        }
        let _bounding_box_drawn = self.draw_freeform_bounding_box(frame, bounds);
        self.draw_freeform_path(frame)
    }

    fn draw_freeform_bounding_box(
        &self,
        frame: &mut canvas::Frame<iced::Renderer>,
        bounds: Rectangle,
    ) -> bool {
        let Some((top_left, size)) = self.calculate_selection_rectangle() else {
            return fill_idle_mask(frame, bounds);
        };
        let _mask_filled = fill_mask_around_selection(
            frame,
            bounds,
            Rectangle::new(top_left, size),
            SELECTION_MASK_COLOR,
        );
        frame.stroke(
            &canvas::Path::rectangle(top_left, size),
            canvas::Stroke::default()
                .with_color(Color {
                    a: FREEFORM_BOUNDING_BOX_ALPHA,
                    ..vibrant::VIOLET
                })
                .with_width(FREEFORM_BOUNDING_BOX_STROKE_WIDTH),
        );
        true
    }

    fn draw_freeform_path(&self, frame: &mut canvas::Frame<iced::Renderer>) -> bool {
        if self.freeform_points.len() <= 1 {
            return false;
        }
        let mut path_builder = canvas::path::Builder::new();
        path_builder.move_to(self.freeform_points[0]);
        for point in self.freeform_points.iter().skip(1) {
            path_builder.line_to(*point);
        }
        if self.is_shape_closed && self.freeform_points.len() > 2 {
            path_builder.close();
        }
        frame.stroke(
            &path_builder.build(),
            canvas::Stroke::default()
                .with_color(vibrant::VIOLET)
                .with_width(FREEFORM_PATH_STROKE_WIDTH),
        );
        true
    }
}
