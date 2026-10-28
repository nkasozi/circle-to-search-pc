use super::*;

const FREEFORM_BOUNDS_PADDING: f32 = 10.0;
const VIEWER_BOUNDS_MISSING_EVENT: &str = "viewer_bounds_missing_using_raw_selection";
const SELECTION_MAPPED_TO_IMAGE_EVENT: &str = "selection_mapped_to_image";

impl CaptureView {
    pub(super) fn calculate_selection_rectangle(&self) -> Option<(Point, Size)> {
        match self.draw_mode {
            DrawMode::Rectangle => {
                rectangle_from_corners(self.selection_start, self.selection_current)
            }
            DrawMode::Freeform => padded_bounding_box(&self.freeform_points),
        }
    }

    pub(super) fn map_selection_to_image(&self, selection_rectangle: Rectangle) -> Rectangle {
        let viewer_bounds = self.viewer_bounds.get();
        if viewer_bounds.width == 0.0 || viewer_bounds.height == 0.0 {
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": VIEWER_BOUNDS_MISSING_EVENT})
            );
            return selection_rectangle;
        }

        let scale_x = self.capture_buffer.width as f32 / viewer_bounds.width;
        let scale_y = self.capture_buffer.height as f32 / viewer_bounds.height;
        let image_x = (selection_rectangle.x - viewer_bounds.x) * scale_x;
        let image_y = (selection_rectangle.y - viewer_bounds.y) * scale_y;
        let image_width = selection_rectangle.width * scale_x;
        let image_height = selection_rectangle.height * scale_y;

        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({
                "event": SELECTION_MAPPED_TO_IMAGE_EVENT,
                "selection_rectangle": format!("{:?}", selection_rectangle),
                "image_x": image_x,
                "image_y": image_y,
                "image_width": image_width,
                "image_height": image_height,
                "viewer_bounds": format!("{:?}", viewer_bounds),
                "capture_buffer_width": self.capture_buffer.width,
                "capture_buffer_height": self.capture_buffer.height,
                "scale_x": scale_x,
                "scale_y": scale_y
            })
        );

        Rectangle::new(
            Point::new(image_x, image_y),
            Size::new(image_width, image_height),
        )
    }
}

fn rectangle_from_corners(start: Option<Point>, current: Option<Point>) -> Option<(Point, Size)> {
    let (Some(start), Some(current)) = (start, current) else {
        return None;
    };
    let x = start.x.min(current.x);
    let y = start.y.min(current.y);
    let width = (start.x - current.x).abs();
    let height = (start.y - current.y).abs();
    Some((Point::new(x, y), Size::new(width, height)))
}

fn padded_bounding_box(points: &[Point]) -> Option<(Point, Size)> {
    if points.is_empty() {
        return None;
    }

    let (minimum_x, minimum_y, maximum_x, maximum_y) = points.iter().fold(
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
        |(minimum_x, minimum_y, maximum_x, maximum_y), point| {
            (
                minimum_x.min(point.x),
                minimum_y.min(point.y),
                maximum_x.max(point.x),
                maximum_y.max(point.y),
            )
        },
    );

    let x = (minimum_x - FREEFORM_BOUNDS_PADDING).max(0.0);
    let y = (minimum_y - FREEFORM_BOUNDS_PADDING).max(0.0);
    let width = maximum_x - minimum_x + (2.0 * FREEFORM_BOUNDS_PADDING);
    let height = maximum_y - minimum_y + (2.0 * FREEFORM_BOUNDS_PADDING);

    Some((Point::new(x, y), Size::new(width, height)))
}
