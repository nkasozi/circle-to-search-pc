use super::*;

const FREEFORM_AUTO_CLOSE_MINIMUM_POINTS: usize = 10;
const FREEFORM_AUTO_CLOSE_DISTANCE: f32 = 20.0;
const FREEFORM_RELEASE_CLOSE_MINIMUM_POINTS: usize = 2;
const FREEFORM_RELEASE_CLOSE_DISTANCE: f32 = 30.0;

impl CaptureView {
    pub(super) fn handle_mouse_pressed(&mut self, point: Point) -> bool {
        match self.draw_mode {
            DrawMode::Rectangle => self.start_rectangle_selection(point),
            DrawMode::Freeform => self.start_freeform_selection(point),
        }
    }

    pub(super) fn handle_mouse_moved(&mut self, point: Point) -> bool {
        if !self.is_selecting {
            return false;
        }
        match self.draw_mode {
            DrawMode::Rectangle => self.extend_rectangle_selection(point),
            DrawMode::Freeform => self.extend_freeform_selection(point),
        }
    }

    pub(super) fn handle_mouse_released(&mut self) -> bool {
        let should_check_shape_closure = self.draw_mode == DrawMode::Freeform
            && self.is_selecting
            && self.freeform_points.len() > FREEFORM_RELEASE_CLOSE_MINIMUM_POINTS;
        if should_check_shape_closure && self.freeform_end_is_near_start() {
            self.is_shape_closed = true;
        }
        self.is_selecting = false;
        true
    }

    pub(super) fn handle_set_draw_mode(&mut self, mode: DrawMode) -> bool {
        self.draw_mode = mode;
        self.reset_selection()
    }

    pub(super) fn reset_selection(&mut self) -> bool {
        self.selection_start = None;
        self.selection_current = None;
        self.is_selecting = false;
        self.freeform_points.clear();
        self.is_shape_closed = false;
        true
    }

    fn start_rectangle_selection(&mut self, point: Point) -> bool {
        self.selection_start = Some(point);
        self.selection_current = Some(point);
        self.is_selecting = true;
        true
    }

    fn start_freeform_selection(&mut self, point: Point) -> bool {
        if self.is_selecting {
            return false;
        }
        self.freeform_points.clear();
        self.freeform_points.push(point);
        self.is_selecting = true;
        self.is_shape_closed = false;
        true
    }

    fn extend_rectangle_selection(&mut self, point: Point) -> bool {
        self.selection_current = Some(point);
        true
    }

    fn extend_freeform_selection(&mut self, point: Point) -> bool {
        self.freeform_points.push(point);
        if self.freeform_points.len() <= FREEFORM_AUTO_CLOSE_MINIMUM_POINTS {
            return true;
        }
        let first_point = self.freeform_points[0];
        let is_near_start = distance_between(point, first_point) < FREEFORM_AUTO_CLOSE_DISTANCE;
        if !is_near_start {
            return true;
        }
        self.is_shape_closed = true;
        self.is_selecting = false;
        true
    }

    fn freeform_end_is_near_start(&self) -> bool {
        let (Some(first_point), Some(last_point)) =
            (self.freeform_points.first(), self.freeform_points.last())
        else {
            return false;
        };
        distance_between(*last_point, *first_point) < FREEFORM_RELEASE_CLOSE_DISTANCE
    }
}

fn distance_between(from: Point, to: Point) -> f32 {
    ((from.x - to.x).powi(2) + (from.y - to.y).powi(2)).sqrt()
}
