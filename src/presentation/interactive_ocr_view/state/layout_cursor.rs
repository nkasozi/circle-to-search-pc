use super::CharPosition;

pub(super) const LINE_HEIGHT_THRESHOLD_RATIO: f32 = 0.5;
const WORD_SPACE_THRESHOLD_RATIO: f32 = 0.3;
const LINE_SEPARATOR: char = '\n';
const WORD_SEPARATOR: char = ' ';

pub(super) struct LayoutCursor {
    last_y: f32,
    last_word_index: usize,
    last_x_end: f32,
}

impl LayoutCursor {
    pub(super) fn starting_at(position: &CharPosition) -> Self {
        Self {
            last_y: position.bounds.y,
            last_word_index: position.word_index,
            last_x_end: right_edge_of(position),
        }
    }
}

pub(super) fn advance_layout_cursor(
    cursor: &LayoutCursor,
    position: &CharPosition,
) -> (Option<char>, LayoutCursor) {
    let starts_new_word = position.word_index != cursor.last_word_index;
    match (starts_new_line(cursor, position), starts_new_word) {
        (true, _) => (Some(LINE_SEPARATOR), LayoutCursor::starting_at(position)),
        (false, true) => (
            word_separator_for(cursor, position),
            LayoutCursor {
                last_y: cursor.last_y,
                last_word_index: position.word_index,
                last_x_end: right_edge_of(position),
            },
        ),
        (false, false) => (
            None,
            LayoutCursor {
                last_y: cursor.last_y,
                last_word_index: cursor.last_word_index,
                last_x_end: right_edge_of(position),
            },
        ),
    }
}

fn starts_new_line(cursor: &LayoutCursor, position: &CharPosition) -> bool {
    let line_height_threshold = position.bounds.height * LINE_HEIGHT_THRESHOLD_RATIO;
    let y_difference = (position.bounds.y - cursor.last_y).abs();
    let position_has_invalid_coordinates =
        !position.bounds.x.is_finite() || !position.bounds.y.is_finite();
    let cursor_has_invalid_coordinates =
        !cursor.last_x_end.is_finite() || !cursor.last_y.is_finite();
    !position_has_invalid_coordinates
        && !cursor_has_invalid_coordinates
        && y_difference > line_height_threshold
}

fn word_separator_for(cursor: &LayoutCursor, position: &CharPosition) -> Option<char> {
    let gap_between_words = position.bounds.x - cursor.last_x_end;
    let space_threshold = position.bounds.width * WORD_SPACE_THRESHOLD_RATIO;
    (gap_between_words > space_threshold).then_some(WORD_SEPARATOR)
}

fn right_edge_of(position: &CharPosition) -> f32 {
    position.bounds.x + position.bounds.width
}
