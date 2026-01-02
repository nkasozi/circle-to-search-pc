use iced::mouse;
use iced::widget::{canvas, container, image, stack, button, row, text};
use iced::{Color, Element, Length, Point, Rectangle, Size, Border, Background, Shadow, Vector, Alignment};

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
            CaptureViewMessage::MousePressed(point) => {
                match self.draw_mode {
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
                }
            }
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
                                let distance = ((point.x - first.x).powi(2) + (point.y - first.y).powi(2)).sqrt();
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
                if self.draw_mode == DrawMode::Freeform && self.is_selecting {
                    if self.freeform_points.len() > 2 {
                        let first = self.freeform_points[0];
                        let last = *self.freeform_points.last().unwrap();
                        let distance = ((last.x - first.x).powi(2) + (last.y - first.y).powi(2)).sqrt();
                        if distance < 30.0 {
                            self.is_shape_closed = true;
                        }
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
        }
    }

    pub fn render_ui(&self) -> Element<'_, CaptureViewMessage> {
        let screenshot_viewer = image::viewer(self.capture_buffer.image_handle.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let overlay_canvas = canvas(self).width(Length::Fill).height(Length::Fill);

        let show_ui = !self.is_selecting;

        let mut layers: Vec<Element<'_, CaptureViewMessage>> = vec![screenshot_viewer.into(), overlay_canvas.into()];

        if show_ui {
            let status_message = if self.calculate_selection_rectangle().is_some() {
                "Press Enter to confirm selection or draw a new region"
            } else {
                match self.draw_mode {
                    DrawMode::Rectangle => "Click and drag to select a region",
                    DrawMode::Freeform => "Click and drag to draw a freeform shape"
                }
            };

            let status_banner = container(
                text(status_message)
                    .size(16)
                    .style(|_theme| {
                        text::Style {
                            color: Some(Color::WHITE),
                        }
                    })
            )
            .padding([12, 24])
            .style(|_theme| {
                container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.85))),
                    border: Border {
                        color: Color::from_rgba(0.3, 0.6, 1.0, 0.8),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                    text_color: None,
                    snap: false,
                }
            });

            let status_positioned = container(status_banner)
                .width(Length::Fill)
                .padding(iced::Padding { top: 80.0, right: 20.0, bottom: 0.0, left: 0.0 })
                .align_x(Alignment::End);

            let rect_btn = button(text("⬜ Rectangle"))
                .padding([8, 16])
                .style(move |theme: &iced::Theme, status| {
                    self.toolbar_button_style(theme, status, self.draw_mode == DrawMode::Rectangle)
                })
                .on_press(CaptureViewMessage::SetDrawMode(DrawMode::Rectangle));

            let freeform_btn = button(text("✏️ Freeform"))
                .padding([8, 16])
                .style(move |theme: &iced::Theme, status| {
                    self.toolbar_button_style(theme, status, self.draw_mode == DrawMode::Freeform)
                })
                .on_press(CaptureViewMessage::SetDrawMode(DrawMode::Freeform));

            let toolbar = container(
                row![rect_btn, freeform_btn]
                    .spacing(8)
                    .padding(8)
            )
            .style(|_theme| {
                container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.85))),
                    border: Border {
                        color: Color::from_rgba(0.4, 0.4, 0.4, 0.9),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                    text_color: None,
                    snap: false,
                }
            });

            let toolbar_positioned = container(toolbar)
                .width(Length::Fill)
                .padding(iced::Padding { top: 80.0, right: 20.0, bottom: 0.0, left: 0.0 })
                .align_x(Alignment::Start);

            layers.push(status_positioned.into());
            layers.push(toolbar_positioned.into());
        }

        let content = stack(layers);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn toolbar_button_style(&self, _theme: &iced::Theme, status: button::Status, is_active: bool) -> button::Style {
        let base_color = if is_active {
            Color::from_rgb(0.3, 0.6, 1.0)
        } else {
            Color::from_rgba(0.4, 0.4, 0.4, 0.8)
        };

        match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(base_color)),
                text_color: Color::WHITE,
                border: Border {
                    color: if is_active {
                        Color::from_rgb(0.4, 0.7, 1.0)
                    } else {
                        Color::from_rgba(0.5, 0.5, 0.5, 0.9)
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(
                    if is_active {
                        Color::from_rgb(0.4, 0.7, 1.0)
                    } else {
                        Color::from_rgba(0.5, 0.5, 0.5, 0.9)
                    }
                )),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::from_rgb(0.5, 0.8, 1.0),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(
                    if is_active {
                        Color::from_rgb(0.2, 0.5, 0.9)
                    } else {
                        Color::from_rgba(0.3, 0.3, 0.3, 0.9)
                    }
                )),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::from_rgb(0.3, 0.6, 0.9),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.5))),
                text_color: Color::from_rgba(0.6, 0.6, 0.6, 1.0),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.5),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
        }
    }

    fn calculate_selection_rectangle(&self) -> Option<(Point, Size)> {
        match self.draw_mode {
            DrawMode::Rectangle => {
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
        let img_width = self.capture_buffer.width as f32;
        let img_height = self.capture_buffer.height as f32;
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

        let viewer_rect = Rectangle::new(
            Point::new(offset_x, offset_y),
            Size::new(display_width, display_height),
        );
        self.viewer_bounds.set(viewer_rect);

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        match self.draw_mode {
            DrawMode::Rectangle => {
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
            }
            DrawMode::Freeform => {
                if !self.freeform_points.is_empty() {
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

                        let bounding_box = canvas::Path::rectangle(top_left, size);
                        frame.stroke(
                            &bounding_box,
                            canvas::Stroke::default()
                                .with_color(Color::from_rgba(0.3, 0.6, 1.0, 0.5))
                                .with_width(1.0),
                        );
                    }

                    if self.freeform_points.len() > 1 {
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
                } else {
                    frame.fill_rectangle(
                        Point::ORIGIN,
                        bounds.size(),
                        Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    );
                }
            }
        }

        vec![frame.into_geometry()]
    }
}
