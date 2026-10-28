use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Pixels, Point, Rectangle, Size};

use crate::presentation::design::tokens::vibrant;

const OUTSIDE_SELECTION_DIM: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.55);
const SELECTION_BORDER_WIDTH: f32 = 2.0;
const HANDLE_SIZE: f32 = 10.0;
const DIMENSION_LABEL_SIZE: f32 = 13.0;
const DIMENSION_LABEL_OFFSET: f32 = 22.0;
const DIMENSION_LABEL_CLEARANCE: f32 = 4.0;
const DIMENSION_LABEL_INSIDE_OFFSET: f32 = 8.0;
const DIMENSION_LABEL_INDENT: f32 = 2.0;
const HANDLE_BORDER_WIDTH: f32 = 2.0;

pub(super) fn draw_selection(frame: &mut Frame, size: Size, selection: Rectangle) -> bool {
    fill_outside(frame, size, selection);
    frame.stroke(
        &Path::rectangle(selection.position(), selection.size()),
        Stroke::default()
            .with_color(vibrant::VIOLET)
            .with_width(SELECTION_BORDER_WIDTH),
    );
    for corner in selection_corners(selection) {
        draw_corner_handle(frame, corner);
    }
    draw_dimension_label(frame, selection)
}

fn selection_corners(selection: Rectangle) -> [Point; 4] {
    [
        Point::new(selection.x, selection.y),
        Point::new(selection.x + selection.width, selection.y),
        Point::new(selection.x, selection.y + selection.height),
        Point::new(
            selection.x + selection.width,
            selection.y + selection.height,
        ),
    ]
}

fn draw_corner_handle(frame: &mut Frame, corner: Point) -> bool {
    let handle_origin = Point::new(corner.x - HANDLE_SIZE / 2.0, corner.y - HANDLE_SIZE / 2.0);
    let handle_size = Size::new(HANDLE_SIZE, HANDLE_SIZE);
    frame.fill_rectangle(handle_origin, handle_size, Color::WHITE);
    frame.stroke(
        &Path::rectangle(handle_origin, handle_size),
        Stroke::default()
            .with_color(vibrant::VIOLET)
            .with_width(HANDLE_BORDER_WIDTH),
    );
    true
}

fn fill_outside(frame: &mut Frame, size: Size, selection: Rectangle) -> bool {
    let selection_bottom = selection.y + selection.height;
    let selection_right = selection.x + selection.width;
    frame.fill_rectangle(
        Point::ORIGIN,
        Size::new(size.width, selection.y.max(0.0)),
        OUTSIDE_SELECTION_DIM,
    );
    frame.fill_rectangle(
        Point::new(0.0, selection_bottom),
        Size::new(size.width, (size.height - selection_bottom).max(0.0)),
        OUTSIDE_SELECTION_DIM,
    );
    frame.fill_rectangle(
        Point::new(0.0, selection.y),
        Size::new(selection.x.max(0.0), selection.height),
        OUTSIDE_SELECTION_DIM,
    );
    frame.fill_rectangle(
        Point::new(selection_right, selection.y),
        Size::new((size.width - selection_right).max(0.0), selection.height),
        OUTSIDE_SELECTION_DIM,
    );
    true
}

fn draw_dimension_label(frame: &mut Frame, selection: Rectangle) -> bool {
    let label_y = match selection.y > DIMENSION_LABEL_OFFSET + DIMENSION_LABEL_CLEARANCE {
        true => selection.y - DIMENSION_LABEL_OFFSET,
        false => selection.y + DIMENSION_LABEL_INSIDE_OFFSET,
    };
    frame.fill_text(Text {
        content: format!("{} × {}", selection.width.round(), selection.height.round()),
        position: Point::new(selection.x + DIMENSION_LABEL_INDENT, label_y),
        color: Color::WHITE,
        size: Pixels(DIMENSION_LABEL_SIZE),
        ..Text::default()
    });
    true
}
