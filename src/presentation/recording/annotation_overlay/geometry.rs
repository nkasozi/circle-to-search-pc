use iced::widget::canvas::{path, Path};
use iced::{Point, Vector};

use crate::presentation::design::tokens::radius;

const ARROW_HEAD_LENGTH: f32 = 22.0;
const ARROW_HEAD_ANGLE: f32 = 0.5;
const SINGLE_POINT_NUDGE: f32 = 0.1;

pub(super) fn build_polyline(points: &[Point]) -> Option<Path> {
    let (first, rest) = points.split_first()?;
    let mut builder = path::Builder::new();
    builder.move_to(*first);
    if rest.is_empty() {
        builder.line_to(Point::new(
            first.x + SINGLE_POINT_NUDGE,
            first.y + SINGLE_POINT_NUDGE,
        ));
    }
    rest.iter().for_each(|point| builder.line_to(*point));
    Some(builder.build())
}

pub(super) fn build_rectangle(points: &[Point]) -> Option<Path> {
    let [start, end] = points else {
        return None;
    };
    let top_left = Point::new(start.x.min(end.x), start.y.min(end.y));
    let size = iced::Size::new((start.x - end.x).abs(), (start.y - end.y).abs());
    Some(Path::rounded_rectangle(
        top_left,
        size,
        radius::STANDARD.into(),
    ))
}

pub(super) fn build_arrow(points: &[Point]) -> Option<Path> {
    let [start, end] = points else {
        return None;
    };
    let direction = Vector::new(end.x - start.x, end.y - start.y);
    let angle = direction.y.atan2(direction.x);
    let head_point = |offset: f32| {
        Point::new(
            end.x - ARROW_HEAD_LENGTH * (angle + offset).cos(),
            end.y - ARROW_HEAD_LENGTH * (angle + offset).sin(),
        )
    };
    let mut builder = path::Builder::new();
    builder.move_to(*start);
    builder.line_to(*end);
    builder.move_to(head_point(ARROW_HEAD_ANGLE));
    builder.line_to(*end);
    builder.line_to(head_point(-ARROW_HEAD_ANGLE));
    Some(builder.build())
}
