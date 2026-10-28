use iced::{Point, Rectangle, Size};

pub(super) struct DisplayLayout {
    image_width: f32,
    image_height: f32,
    display_width: f32,
    display_height: f32,
    offset_x: f32,
    offset_y: f32,
    scale_x: f32,
    scale_y: f32,
}

impl DisplayLayout {
    pub(super) fn fit(image_width: f32, image_height: f32, bounds: Rectangle) -> Self {
        let image_aspect = image_width / image_height;
        let bounds_aspect = bounds.width / bounds.height;
        let (display_width, display_height, offset_x, offset_y) = match image_aspect > bounds_aspect
        {
            true => {
                let display_height = bounds.width / image_aspect;
                let offset_y = (bounds.height - display_height) / 2.0;
                (bounds.width, display_height, 0.0, offset_y)
            }
            false => {
                let display_width = bounds.height * image_aspect;
                let offset_x = (bounds.width - display_width) / 2.0;
                (display_width, bounds.height, offset_x, 0.0)
            }
        };
        Self {
            image_width,
            image_height,
            display_width,
            display_height,
            offset_x,
            offset_y,
            scale_x: display_width / image_width,
            scale_y: display_height / image_height,
        }
    }

    pub(super) fn to_canvas_rectangle(&self, image_rectangle: &Rectangle) -> Rectangle {
        Rectangle::new(
            Point::new(
                self.offset_x + (image_rectangle.x * self.scale_x),
                self.offset_y + (image_rectangle.y * self.scale_y),
            ),
            Size::new(
                image_rectangle.width * self.scale_x,
                image_rectangle.height * self.scale_y,
            ),
        )
    }

    pub(super) fn to_canvas_stroke_point(&self, image_point: Point) -> Point {
        Point::new(
            self.offset_x + (image_point.x / self.image_width) * self.display_width,
            self.offset_y + (image_point.y / self.image_height) * self.display_height,
        )
    }

    pub(super) fn to_image_point(&self, canvas_point: Point) -> Point {
        Point::new(
            (canvas_point.x - self.offset_x) / self.scale_x,
            (canvas_point.y - self.offset_y) / self.scale_y,
        )
    }
}
