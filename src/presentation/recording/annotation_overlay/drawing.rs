use iced::widget::canvas::{Frame, LineCap, LineDash, LineJoin, Path, Stroke};
use iced::{Color, Point, Rectangle};

use super::geometry::{build_arrow, build_polyline, build_rectangle};
use super::ScreenAnnotationView;
use crate::presentation::design::tokens::vibrant;
use crate::presentation::recording::annotation::{AnnotationStroke, AnnotationTool};
use crate::presentation::recording::pointer_effects::ClickRipple;

const PRESSED_DOT_RADIUS: f32 = 13.0;
const PRESSED_DOT_COLOR: Color = Color::from_rgba(1.0, 0.84, 0.0, 0.55);
const RIPPLE_STROKE_WIDTH: f32 = 4.0;
const SPOTLIGHT_RADIUS: f32 = 30.0;
const SPOTLIGHT_COLOR: Color = Color::from_rgba(1.0, 0.84, 0.0, 0.22);
const REGION_FRAME_OFFSET: f32 = 3.0;
const REGION_FRAME_WIDTH: f32 = 2.0;
const REGION_FRAME_DASH: [f32; 2] = [10.0, 6.0];

pub(super) fn draw_pointer_effects(frame: &mut Frame, view: &ScreenAnnotationView) -> bool {
    let (Some(cursor), Some(now)) = (view.pointer.cursor, view.pointer.now) else {
        return false;
    };
    if view.cursor_spotlight_enabled {
        frame.fill(&Path::circle(cursor, SPOTLIGHT_RADIUS), SPOTLIGHT_COLOR);
    }
    if !view.click_highlights_enabled {
        return true;
    }
    if view.pointer.is_pressed {
        frame.fill(&Path::circle(cursor, PRESSED_DOT_RADIUS), PRESSED_DOT_COLOR);
    }
    for ripple in &view.pointer.ripples {
        draw_ripple(frame, ripple, now);
    }
    true
}

fn draw_ripple(frame: &mut Frame, ripple: &ClickRipple, now: std::time::Instant) -> bool {
    let Some(appearance) = ripple.appearance(now) else {
        return false;
    };
    let ring_color = Color {
        a: appearance.opacity,
        ..vibrant::YELLOW
    };
    frame.stroke(
        &Path::circle(ripple.center, appearance.radius),
        Stroke::default()
            .with_color(ring_color)
            .with_width(RIPPLE_STROKE_WIDTH),
    );
    true
}

pub(super) fn draw_region_frame(frame: &mut Frame, region: Rectangle) -> bool {
    let outline = Path::rectangle(
        Point::new(
            region.x - REGION_FRAME_OFFSET,
            region.y - REGION_FRAME_OFFSET,
        ),
        iced::Size::new(
            region.width + REGION_FRAME_OFFSET * 2.0,
            region.height + REGION_FRAME_OFFSET * 2.0,
        ),
    );
    let stroke = Stroke {
        line_dash: LineDash {
            segments: &REGION_FRAME_DASH,
            offset: 0,
        },
        ..Stroke::default()
            .with_color(vibrant::RED)
            .with_width(REGION_FRAME_WIDTH)
    };
    frame.stroke(&outline, stroke);
    true
}

pub(super) fn draw_stroke(frame: &mut Frame, stroke: &AnnotationStroke) -> bool {
    let style = Stroke::default()
        .with_color(stroke.tool.stroke_color(stroke.color))
        .with_width(stroke.tool.stroke_width())
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round);
    let shape = match stroke.tool {
        AnnotationTool::Pen | AnnotationTool::Highlighter => build_polyline(&stroke.points),
        AnnotationTool::Arrow => build_arrow(&stroke.points),
        AnnotationTool::Rectangle => build_rectangle(&stroke.points),
    };
    let Some(shape_path) = shape else {
        return false;
    };
    frame.stroke(&shape_path, style);
    true
}
