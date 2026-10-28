use super::geometry::{build_arrow, build_polyline, build_rectangle};
use iced::Point;

#[test]
fn test_arrow_and_rectangle_need_two_points() {
    assert!(build_arrow(&[Point::ORIGIN]).is_none());
    assert!(build_rectangle(&[Point::ORIGIN, Point::new(10.0, 10.0)]).is_some());
    assert!(build_polyline(&[]).is_none());
    assert!(build_polyline(&[Point::ORIGIN]).is_some());
}
