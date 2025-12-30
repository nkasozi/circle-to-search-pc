use iced::mouse;
use iced::widget::{canvas, container, image, stack};
use iced::{Color, Element, Length, Point, Rectangle, Size};

use crate::core::models::CaptureBuffer;

pub struct CaptureView {
    capture_buffer: CaptureBuffer,
    selection_start: Option<Point>,
    selection_current: Option<Point>,
    is_selecting: bool,
}

#[derive(Debug, Clone)]
pub enum CaptureViewMessage {
    MousePressed(Point),
    MouseMoved(Point),
    MouseReleased,
    ConfirmSelection,
    #[allow(dead_code)]
    CancelRequested,
}

impl CaptureView {
    pub fn build_with_capture_buffer(capture_buffer: CaptureBuffer) -> Self {
        log::debug!("[CAPTURE_VIEW] building view");
        Self {
            capture_buffer,
            selection_start: None,
            selection_current: None,
            is_selecting: false,
        }
    }

    pub fn update(&mut self, message: CaptureViewMessage) {
        match message {
            CaptureViewMessage::MousePressed(point) => {
                self.selection_start = Some(point);
                self.selection_current = Some(point);
                self.is_selecting = true;
            }
            CaptureViewMessage::MouseMoved(point) => {
                if self.is_selecting {
                    self.selection_current = Some(point);
                }
            }
            CaptureViewMessage::MouseReleased => {
                self.is_selecting = false;
            }
            CaptureViewMessage::ConfirmSelection => {}
            CaptureViewMessage::CancelRequested => {
                self.selection_start = None;
                self.selection_current = None;
                self.is_selecting = false;
            }
        }
    }

    pub fn render_ui(&self) -> Element<'_, CaptureViewMessage> {
        let screenshot_viewer = image::viewer(self.capture_buffer.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let overlay_canvas = canvas(self).width(Length::Fill).height(Length::Fill);

        let content = stack![screenshot_viewer, overlay_canvas];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn calculate_selection_rectangle(&self) -> Option<(Point, Size)> {
        match (self.selection_start, self.selection_current) {
            (Some(start), Some(current)) => {
                let x = start.x.min(current.x);
                let y = start.y.min(current.y);
                let width = (start.x - current.x).abs();
                let height = (start.y - current.y).abs();
                Some((Point::new(x, y), Size::new(width, height)))
            }
            _ => None,
        }
    }
}

impl canvas::Program<CaptureViewMessage> for CaptureView {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<CaptureViewMessage>> {
        match event {
            iced::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let mouse::Cursor::Available(position) = _cursor {
                        Some(canvas::Action::publish(CaptureViewMessage::MousePressed(
                            position,
                        )))
                    } else {
                        None
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if let mouse::Cursor::Available(position) = _cursor {
                        Some(canvas::Action::publish(CaptureViewMessage::MouseMoved(
                            position,
                        )))
                    } else {
                        None
                    }
                }
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
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        if let Some((top_left, size)) = self.calculate_selection_rectangle() {
            let overlay_color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);

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

            let selection_path = canvas::Path::rectangle(top_left, size);
            frame.stroke(
                &selection_path,
                canvas::Stroke::default()
                    .with_color(Color::from_rgb(0.3, 0.6, 1.0))
                    .with_width(2.0),
            );
        } else {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            );
        }

        vec![frame.into_geometry()]
    }
}
