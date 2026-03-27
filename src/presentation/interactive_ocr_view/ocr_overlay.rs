use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

use super::{CharPosition, DrawStroke, InteractiveOcrMessage};

pub(super) struct OcrOverlay {
    pub(super) char_positions: Vec<CharPosition>,
    pub(super) image_width: u32,
    pub(super) image_height: u32,
    pub(super) selected_indices: Vec<usize>,
    pub(super) draw_strokes: Vec<DrawStroke>,
    pub(super) current_stroke_points: Vec<Point>,
    pub(super) is_drawing: bool,
    pub(super) draw_color: Color,
    pub(super) draw_width: f32,
    pub(super) draw_mode_enabled: bool,
    pub(super) draw_panel_position: Point,
    pub(super) draw_panel_is_dragging: bool,
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
                    Some(canvas::Action::publish(InteractiveOcrMessage::Close))
                }
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Character(c),
                    modifiers,
                    ..
                } => {
                    let char_str = c.as_str();
                    let is_cmd_or_ctrl = modifiers.command() || modifiers.control();

                    if is_cmd_or_ctrl && char_str == "a" {
                        log::debug!("[INTERACTIVE_OCR] Select all triggered via keyboard shortcut");
                        return Some(canvas::Action::publish(InteractiveOcrMessage::SelectAll));
                    }
                    if is_cmd_or_ctrl && char_str == "c" {
                        log::debug!("[INTERACTIVE_OCR] Copy text triggered via keyboard shortcut");
                        return Some(canvas::Action::publish(InteractiveOcrMessage::CopySelected));
                    }
                    if is_cmd_or_ctrl && char_str == "s" {
                        log::debug!("[INTERACTIVE_OCR] Save image triggered via keyboard shortcut");
                        return Some(canvas::Action::publish(
                            InteractiveOcrMessage::SaveImageToFile,
                        ));
                    }
                    if is_cmd_or_ctrl && char_str == "d" {
                        log::debug!("[INTERACTIVE_OCR] Copy image triggered via keyboard shortcut");
                        return Some(canvas::Action::publish(
                            InteractiveOcrMessage::CopyImageToClipboard,
                        ));
                    }
                    None
                }
                _ => None,
            },
            iced::Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    let Some(cursor_position) = cursor.position_in(bounds) else {
                        return None;
                    };
                    let handle_strip_rect =
                        Rectangle::new(self.draw_panel_position, Size::new(300.0, 26.0));
                    if handle_strip_rect.contains(cursor_position) {
                        log::debug!(
                            "[OCR_OVERLAY] Draw panel drag started at ({}, {})",
                            cursor_position.x,
                            cursor_position.y
                        );
                        return Some(canvas::Action::publish(
                            InteractiveOcrMessage::DrawPanelDragStarted(
                                cursor_position.x,
                                cursor_position.y,
                            ),
                        ));
                    }
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

                    None
                }
                iced::mouse::Event::CursorMoved { .. } => {
                    let Some(cursor_position) = cursor.position_in(bounds) else {
                        return None;
                    };
                    if self.draw_panel_is_dragging {
                        return Some(canvas::Action::publish(
                            InteractiveOcrMessage::DrawPanelMoved(
                                cursor_position.x,
                                cursor_position.y,
                            ),
                        ));
                    }
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

                    None
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    if self.draw_panel_is_dragging {
                        return Some(canvas::Action::publish(
                            InteractiveOcrMessage::DrawPanelReleased,
                        ));
                    }
                    if self.is_drawing {
                        return Some(canvas::Action::publish(InteractiveOcrMessage::EndDrawing));
                    }
                    Some(canvas::Action::publish(InteractiveOcrMessage::EndDrag))
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right) => {
                    Some(canvas::Action::publish(InteractiveOcrMessage::EndDrawing))
                }
                _ => None,
            },
            _ => None,
        }
    }
}
