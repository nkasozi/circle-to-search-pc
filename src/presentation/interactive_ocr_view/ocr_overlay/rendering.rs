use iced::widget::canvas;
use iced::{Color, Point};

use super::{DisplayLayout, OcrOverlay};

const SELECTED_CHARACTER_FILL: Color = Color::from_rgba(0.3, 0.8, 0.3, 0.4);
const UNSELECTED_CHARACTER_FILL: Color = Color::from_rgba(0.2, 0.6, 1.0, 0.15);
const SELECTED_CHARACTER_BORDER: Color = Color::from_rgb(0.2, 0.9, 0.2);
const SELECTED_CHARACTER_BORDER_WIDTH: f32 = 1.5;

type OverlayFrame = canvas::Frame<iced::Renderer>;

impl OcrOverlay {
    pub(super) fn draw_character_highlights(
        &self,
        frame: &mut OverlayFrame,
        display_layout: &DisplayLayout,
    ) -> bool {
        for (index, character_position) in self.char_positions.iter().enumerate() {
            let canvas_rectangle = display_layout.to_canvas_rectangle(&character_position.bounds);
            let is_selected = self.selected_indices.contains(&index);
            let fill_color = match is_selected {
                true => SELECTED_CHARACTER_FILL,
                false => UNSELECTED_CHARACTER_FILL,
            };
            frame.fill_rectangle(
                canvas_rectangle.position(),
                canvas_rectangle.size(),
                fill_color,
            );
            if is_selected {
                frame.stroke(
                    &canvas::Path::rectangle(canvas_rectangle.position(), canvas_rectangle.size()),
                    canvas::Stroke::default()
                        .with_color(SELECTED_CHARACTER_BORDER)
                        .with_width(SELECTED_CHARACTER_BORDER_WIDTH),
                );
            }
        }
        !self.char_positions.is_empty()
    }

    pub(super) fn draw_saved_strokes(
        &self,
        frame: &mut OverlayFrame,
        display_layout: &DisplayLayout,
    ) -> bool {
        self.draw_strokes.iter().fold(false, |any_drawn, stroke| {
            let stroke_drawn = draw_stroke_path(
                frame,
                display_layout,
                &stroke.points,
                stroke.color,
                stroke.width,
            );
            any_drawn || stroke_drawn
        })
    }

    pub(super) fn draw_current_stroke(
        &self,
        frame: &mut OverlayFrame,
        display_layout: &DisplayLayout,
    ) -> bool {
        if !self.is_drawing {
            return false;
        }
        draw_stroke_path(
            frame,
            display_layout,
            &self.current_stroke_points,
            self.draw_color,
            self.draw_width,
        )
    }
}

fn draw_stroke_path(
    frame: &mut OverlayFrame,
    display_layout: &DisplayLayout,
    points: &[Point],
    color: Color,
    width: f32,
) -> bool {
    let Some((first_point, remaining_points)) = points.split_first() else {
        return false;
    };
    if remaining_points.is_empty() {
        return false;
    }
    let mut path_builder = canvas::path::Builder::new();
    path_builder.move_to(display_layout.to_canvas_stroke_point(*first_point));
    for point in remaining_points {
        path_builder.line_to(display_layout.to_canvas_stroke_point(*point));
    }
    frame.stroke(
        &path_builder.build(),
        canvas::Stroke::default()
            .with_color(color)
            .with_width(width),
    );
    true
}
