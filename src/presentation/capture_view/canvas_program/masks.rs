use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

const IDLE_MASK_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.3);

pub(super) fn fit_image_in_bounds(
    image_width: f32,
    image_height: f32,
    bounds: Rectangle,
) -> Rectangle {
    let image_aspect = image_width / image_height;
    let bounds_aspect = bounds.width / bounds.height;
    match image_aspect > bounds_aspect {
        true => {
            let display_height = bounds.width / image_aspect;
            Rectangle::new(
                Point::new(0.0, (bounds.height - display_height) / 2.0),
                Size::new(bounds.width, display_height),
            )
        }
        false => {
            let display_width = bounds.height * image_aspect;
            Rectangle::new(
                Point::new((bounds.width - display_width) / 2.0, 0.0),
                Size::new(display_width, bounds.height),
            )
        }
    }
}

pub(super) fn fill_idle_mask(frame: &mut canvas::Frame<iced::Renderer>, bounds: Rectangle) -> bool {
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), IDLE_MASK_COLOR);
    true
}

pub(super) fn fill_mask_around_selection(
    frame: &mut canvas::Frame<iced::Renderer>,
    bounds: Rectangle,
    selection: Rectangle,
    overlay_color: Color,
) -> bool {
    let selection_bottom = selection.y + selection.height;
    let selection_right = selection.x + selection.width;
    frame.fill_rectangle(
        Point::new(0.0, 0.0),
        Size::new(bounds.width, selection.y),
        overlay_color,
    );
    frame.fill_rectangle(
        Point::new(0.0, selection_bottom),
        Size::new(bounds.width, bounds.height - selection_bottom),
        overlay_color,
    );
    frame.fill_rectangle(
        Point::new(0.0, selection.y),
        Size::new(selection.x, selection.height),
        overlay_color,
    );
    frame.fill_rectangle(
        Point::new(selection_right, selection.y),
        Size::new(bounds.width - selection_right, selection.height),
        overlay_color,
    );
    true
}
