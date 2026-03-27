use super::*;
use iced::mouse;
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

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
        let img_width = self.capture_buffer.width as f32;
        let img_height = self.capture_buffer.height as f32;
        let img_aspect = img_width / img_height;
        let bounds_aspect = bounds.width / bounds.height;
        let (display_width, display_height, offset_x, offset_y) = if img_aspect > bounds_aspect {
            let display_width = bounds.width;
            let display_height = bounds.width / img_aspect;
            (
                display_width,
                display_height,
                0.0,
                (bounds.height - display_height) / 2.0,
            )
        } else {
            let display_height = bounds.height;
            let display_width = bounds.height * img_aspect;
            (
                display_width,
                display_height,
                (bounds.width - display_width) / 2.0,
                0.0,
            )
        };

        self.viewer_bounds.set(Rectangle::new(
            Point::new(offset_x, offset_y),
            Size::new(display_width, display_height),
        ));
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        match self.draw_mode {
            DrawMode::Rectangle => self.draw_rectangle_overlay(&mut frame, bounds),
            DrawMode::Freeform => self.draw_freeform_overlay(&mut frame, bounds),
        }

        vec![frame.into_geometry()]
    }
}

impl CaptureView {
    fn draw_rectangle_overlay(&self, frame: &mut canvas::Frame<iced::Renderer>, bounds: Rectangle) {
        match self.calculate_selection_rectangle() {
            Some((top_left, size)) => {
                self.fill_mask_around_selection(
                    frame,
                    bounds,
                    top_left,
                    size,
                    Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                );
                let selection_path = canvas::Path::rectangle(top_left, size);
                frame.stroke(
                    &selection_path,
                    canvas::Stroke::default()
                        .with_color(Color::from_rgb(0.3, 0.6, 1.0))
                        .with_width(2.0),
                );
            }
            None => frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            ),
        }
    }

    fn draw_freeform_overlay(&self, frame: &mut canvas::Frame<iced::Renderer>, bounds: Rectangle) {
        if self.freeform_points.is_empty() {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            );
            return;
        }

        match self.calculate_selection_rectangle() {
            Some((top_left, size)) => {
                self.fill_mask_around_selection(
                    frame,
                    bounds,
                    top_left,
                    size,
                    Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                );
                let bounding_box = canvas::Path::rectangle(top_left, size);
                frame.stroke(
                    &bounding_box,
                    canvas::Stroke::default()
                        .with_color(Color::from_rgba(0.3, 0.6, 1.0, 0.5))
                        .with_width(1.0),
                );
            }
            None => frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            ),
        }

        if self.freeform_points.len() <= 1 {
            return;
        }
        let mut path_builder = canvas::path::Builder::new();
        path_builder.move_to(self.freeform_points[0]);
        for point in self.freeform_points.iter().skip(1) {
            path_builder.line_to(*point);
        }
        if self.is_shape_closed && self.freeform_points.len() > 2 {
            path_builder.close();
        }
        let path = path_builder.build();
        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_color(Color::from_rgb(0.3, 0.6, 1.0))
                .with_width(3.0),
        );
    }

    fn fill_mask_around_selection(
        &self,
        frame: &mut canvas::Frame<iced::Renderer>,
        bounds: Rectangle,
        top_left: Point,
        size: Size,
        overlay_color: Color,
    ) {
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            Size::new(bounds.width, top_left.y),
            overlay_color,
        );
        frame.fill_rectangle(
            Point::new(0.0, top_left.y + size.height),
            Size::new(bounds.width, bounds.height - (top_left.y + size.height)),
            overlay_color,
        );
        frame.fill_rectangle(
            Point::new(0.0, top_left.y),
            Size::new(top_left.x, size.height),
            overlay_color,
        );
        frame.fill_rectangle(
            Point::new(top_left.x + size.width, top_left.y),
            Size::new(bounds.width - (top_left.x + size.width), size.height),
            overlay_color,
        );
    }
}
