use iced::{Color, Point};

use crate::presentation::design::tokens::vibrant;

pub const ANNOTATION_PALETTE: [Color; 6] = [
    vibrant::RED,
    vibrant::YELLOW,
    vibrant::GREEN,
    vibrant::BLUE,
    vibrant::PINK,
    vibrant::WHITE,
];

const PEN_WIDTH: f32 = 5.0;
const HIGHLIGHTER_WIDTH: f32 = 22.0;
const SHAPE_WIDTH: f32 = 5.0;
const HIGHLIGHTER_ALPHA: f32 = 0.38;
const MINIMUM_POINT_SPACING: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnotationTool {
    Pen,
    Highlighter,
    Arrow,
    Rectangle,
}

impl AnnotationTool {
    pub fn stroke_width(self) -> f32 {
        match self {
            AnnotationTool::Pen => PEN_WIDTH,
            AnnotationTool::Highlighter => HIGHLIGHTER_WIDTH,
            AnnotationTool::Arrow | AnnotationTool::Rectangle => SHAPE_WIDTH,
        }
    }

    pub fn stroke_color(self, base_color: Color) -> Color {
        match self {
            AnnotationTool::Highlighter => Color {
                a: HIGHLIGHTER_ALPHA,
                ..base_color
            },
            _ => base_color,
        }
    }

    pub fn is_two_point_shape(self) -> bool {
        matches!(self, AnnotationTool::Arrow | AnnotationTool::Rectangle)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationStroke {
    pub tool: AnnotationTool,
    pub color: Color,
    pub points: Vec<Point>,
}

impl AnnotationStroke {
    pub fn begin(tool: AnnotationTool, color: Color, origin: Point) -> Self {
        Self {
            tool,
            color,
            points: vec![origin],
        }
    }

    pub fn extend(mut self, point: Point) -> Self {
        if self.tool.is_two_point_shape() {
            self.points.truncate(1);
            self.points.push(point);
            return self;
        }
        let is_far_enough = self
            .points
            .last()
            .map(|last| last.distance(point) >= MINIMUM_POINT_SPACING)
            .unwrap_or(true);
        if is_far_enough {
            self.points.push(point);
        }
        self
    }

    pub fn is_visible(&self) -> bool {
        match self.tool.is_two_point_shape() {
            true => self.points.len() == 2 && self.points[0].distance(self.points[1]) > 4.0,
            false => !self.points.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationBoard {
    pub strokes: Vec<AnnotationStroke>,
    pub active_stroke: Option<AnnotationStroke>,
    pub tool: AnnotationTool,
    pub color: Color,
}

impl Default for AnnotationBoard {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            active_stroke: None,
            tool: AnnotationTool::Pen,
            color: ANNOTATION_PALETTE[0],
        }
    }
}

impl AnnotationBoard {
    pub fn start_stroke(mut self, origin: Point) -> Self {
        self.active_stroke = Some(AnnotationStroke::begin(self.tool, self.color, origin));
        self
    }

    pub fn extend_stroke(mut self, point: Point) -> Self {
        self.active_stroke = self.active_stroke.take().map(|stroke| stroke.extend(point));
        self
    }

    pub fn finish_stroke(mut self) -> Self {
        if let Some(stroke) = self
            .active_stroke
            .take()
            .filter(AnnotationStroke::is_visible)
        {
            self.strokes.push(stroke);
        }
        self
    }

    pub fn undo(mut self) -> Self {
        self.strokes.pop();
        self
    }

    pub fn clear(mut self) -> Self {
        self.strokes.clear();
        self.active_stroke = None;
        self
    }
}

#[cfg(test)]
mod tests;
