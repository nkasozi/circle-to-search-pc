use super::*;

fn build_view() -> CaptureView {
    CaptureView::build_with_capture_buffer(CaptureBuffer::build_from_raw_data(
        1.0,
        10,
        10,
        vec![0; 400],
    ))
}

fn drag(view: &mut CaptureView, from: Point, to: Point) -> bool {
    view.update(CaptureViewMessage::MousePressed(from));
    view.update(CaptureViewMessage::MouseMoved(to));
    view.update(CaptureViewMessage::MouseReleased);
    view.has_completed_selection()
}

#[test]
fn test_rectangle_drag_completes_selection_on_release() {
    let mut view = build_view();
    drag(&mut view, Point::new(10.0, 10.0), Point::new(120.0, 90.0));
    assert!(view.has_completed_selection());
}

#[test]
fn test_tiny_click_does_not_complete_selection() {
    let mut view = build_view();
    drag(&mut view, Point::new(10.0, 10.0), Point::new(12.0, 11.0));
    assert!(!view.has_completed_selection());
}

#[test]
fn test_selection_is_not_complete_while_dragging() {
    let mut view = build_view();
    view.update(CaptureViewMessage::MousePressed(Point::new(0.0, 0.0)));
    view.update(CaptureViewMessage::MouseMoved(Point::new(100.0, 100.0)));
    assert!(!view.has_completed_selection());
}

#[test]
fn test_freeform_circle_completes_selection() {
    let mut view = build_view();
    view.update(CaptureViewMessage::SetDrawMode(DrawMode::Freeform));
    view.update(CaptureViewMessage::MousePressed(Point::new(50.0, 10.0)));
    for (x, y) in [
        (90.0, 30.0),
        (100.0, 60.0),
        (80.0, 95.0),
        (40.0, 100.0),
        (10.0, 70.0),
        (15.0, 30.0),
        (48.0, 12.0),
    ] {
        view.update(CaptureViewMessage::MouseMoved(Point::new(x, y)));
    }
    view.update(CaptureViewMessage::MouseReleased);
    assert!(view.has_completed_selection());
}
