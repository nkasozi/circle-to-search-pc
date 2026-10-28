use super::*;

#[test]
fn test_pen_stroke_collects_points_and_is_committed() {
    let board = AnnotationBoard::default()
        .start_stroke(Point::new(0.0, 0.0))
        .extend_stroke(Point::new(10.0, 0.0))
        .extend_stroke(Point::new(10.5, 0.0))
        .finish_stroke();
    assert_eq!(board.strokes.len(), 1);
    assert_eq!(board.strokes[0].points.len(), 2);
    assert!(board.active_stroke.is_none());
}

#[test]
fn test_arrow_keeps_only_start_and_end() {
    let board = AnnotationBoard {
        tool: AnnotationTool::Arrow,
        ..AnnotationBoard::default()
    }
    .start_stroke(Point::new(0.0, 0.0))
    .extend_stroke(Point::new(5.0, 5.0))
    .extend_stroke(Point::new(50.0, 20.0))
    .finish_stroke();
    assert_eq!(
        board.strokes[0].points,
        vec![Point::new(0.0, 0.0), Point::new(50.0, 20.0)]
    );
}

#[test]
fn test_tiny_shapes_are_discarded() {
    let board = AnnotationBoard {
        tool: AnnotationTool::Rectangle,
        ..AnnotationBoard::default()
    }
    .start_stroke(Point::new(0.0, 0.0))
    .extend_stroke(Point::new(1.0, 1.0))
    .finish_stroke();
    assert!(board.strokes.is_empty());
}

#[test]
fn test_undo_and_clear() {
    let board = AnnotationBoard::default()
        .start_stroke(Point::ORIGIN)
        .finish_stroke()
        .start_stroke(Point::ORIGIN)
        .finish_stroke();
    assert_eq!(board.clone().undo().strokes.len(), 1);
    assert!(board.clear().strokes.is_empty());
}

#[test]
fn test_highlighter_is_translucent_and_wide() {
    let color = AnnotationTool::Highlighter.stroke_color(vibrant::YELLOW);
    assert!(color.a < 0.5);
    assert!(AnnotationTool::Highlighter.stroke_width() > AnnotationTool::Pen.stroke_width());
}
