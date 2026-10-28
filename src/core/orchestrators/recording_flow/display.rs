use iced::{Point, Rectangle, Size};

use crate::core::models::{CaptureArea, LogicalRect};

const CAMERA_BUBBLE_MARGIN: f32 = 28.0;
const CONTROLS_BOTTOM_MARGIN: f32 = 96.0;
const READY_CARD_MARGIN: f32 = 24.0;
const READY_CARD_BOTTOM_MARGIN: f32 = 96.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayGeometry {
    pub origin: Point,
    pub size: Size,
    pub scale_factor: f64,
}

impl DisplayGeometry {
    pub fn from_raw_monitor(
        raw_origin: (i32, i32),
        raw_size: (u32, u32),
        scale_factor: f64,
        values_are_physical: bool,
    ) -> Self {
        let divisor = match values_are_physical {
            true => scale_factor.max(1.0) as f32,
            false => 1.0,
        };
        Self {
            origin: Point::new(raw_origin.0 as f32 / divisor, raw_origin.1 as f32 / divisor),
            size: Size::new(raw_size.0 as f32 / divisor, raw_size.1 as f32 / divisor),
            scale_factor,
        }
    }

    pub fn to_global(&self, local_point: Point) -> Point {
        Point::new(self.origin.x + local_point.x, self.origin.y + local_point.y)
    }

    pub fn to_local(&self, global_x: f64, global_y: f64) -> Option<Point> {
        let local = Point::new(
            global_x as f32 - self.origin.x,
            global_y as f32 - self.origin.y,
        );
        let is_inside = local.x >= 0.0
            && local.y >= 0.0
            && local.x <= self.size.width
            && local.y <= self.size.height;
        is_inside.then_some(local)
    }

    pub fn full_rectangle(&self) -> Rectangle {
        Rectangle::new(Point::ORIGIN, self.size)
    }

    pub fn capture_area_for(&self, local_rectangle: Rectangle) -> CaptureArea {
        if local_rectangle == self.full_rectangle() {
            return CaptureArea::FullDisplay;
        }
        CaptureArea::Region(LogicalRect {
            x: (self.origin.x + local_rectangle.x) as f64,
            y: (self.origin.y + local_rectangle.y) as f64,
            width: local_rectangle.width as f64,
            height: local_rectangle.height as f64,
        })
    }

    pub fn camera_bubble_position(
        &self,
        local_capture_rectangle: Rectangle,
        bubble_edge: f32,
    ) -> Point {
        let local_position = Point::new(
            local_capture_rectangle.x + local_capture_rectangle.width
                - bubble_edge
                - CAMERA_BUBBLE_MARGIN,
            local_capture_rectangle.y + local_capture_rectangle.height
                - bubble_edge
                - CAMERA_BUBBLE_MARGIN,
        );
        self.to_global(Point::new(
            local_position.x.max(local_capture_rectangle.x),
            local_position.y.max(local_capture_rectangle.y),
        ))
    }

    pub fn controls_position(&self, controls_size: Size) -> Point {
        self.to_global(Point::new(
            (self.size.width - controls_size.width) / 2.0,
            self.size.height - controls_size.height - CONTROLS_BOTTOM_MARGIN,
        ))
    }

    pub fn ready_card_position(&self, card_size: Size) -> Point {
        self.to_global(Point::new(
            self.size.width - card_size.width - READY_CARD_MARGIN,
            self.size.height - card_size.height - READY_CARD_BOTTOM_MARGIN,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display() -> DisplayGeometry {
        DisplayGeometry {
            origin: Point::new(1440.0, 0.0),
            size: Size::new(1920.0, 1080.0),
            scale_factor: 2.0,
        }
    }

    #[test]
    fn test_physical_monitor_values_are_converted_to_logical() {
        let geometry = DisplayGeometry::from_raw_monitor((2880, 0), (3840, 2160), 2.0, true);
        assert_eq!(geometry.origin, Point::new(1440.0, 0.0));
        assert_eq!(geometry.size, Size::new(1920.0, 1080.0));
    }

    #[test]
    fn test_local_and_global_conversion() {
        assert_eq!(
            display().to_global(Point::new(10.0, 20.0)),
            Point::new(1450.0, 20.0)
        );
        assert_eq!(
            display().to_local(1450.0, 20.0),
            Some(Point::new(10.0, 20.0))
        );
        assert_eq!(display().to_local(100.0, 20.0), None);
    }

    #[test]
    fn test_full_rectangle_maps_to_full_display_capture() {
        assert_eq!(
            display().capture_area_for(display().full_rectangle()),
            CaptureArea::FullDisplay
        );
        let region = display().capture_area_for(Rectangle {
            x: 100.0,
            y: 50.0,
            width: 400.0,
            height: 300.0,
        });
        assert_eq!(
            region,
            CaptureArea::Region(LogicalRect {
                x: 1540.0,
                y: 50.0,
                width: 400.0,
                height: 300.0
            })
        );
    }

    #[test]
    fn test_camera_bubble_sits_inside_bottom_right_of_capture_area() {
        let position = display().camera_bubble_position(
            Rectangle {
                x: 100.0,
                y: 100.0,
                width: 800.0,
                height: 600.0,
            },
            200.0,
        );
        assert_eq!(position, Point::new(1440.0 + 672.0, 472.0));
    }

    #[test]
    fn test_camera_bubble_never_leaves_small_regions() {
        let position = display().camera_bubble_position(
            Rectangle {
                x: 100.0,
                y: 100.0,
                width: 120.0,
                height: 120.0,
            },
            200.0,
        );
        assert_eq!(position, Point::new(1540.0, 100.0));
    }

    #[test]
    fn test_controls_are_centered_near_bottom() {
        let position = display().controls_position(Size::new(460.0, 120.0));
        assert_eq!(position, Point::new(1440.0 + 730.0, 864.0));
    }
}
