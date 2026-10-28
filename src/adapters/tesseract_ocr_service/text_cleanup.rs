use super::ocr_replacements::OCR_REPLACEMENTS;

const EMPTY_BOUNDS: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.0);

pub(super) fn cleanup_ocr_artifacts(text: &str) -> String {
    let replaced_text = OCR_REPLACEMENTS
        .iter()
        .fold(text.to_string(), |current_text, (pattern, replacement)| {
            current_text.replace(pattern, replacement)
        });
    replaced_text.trim().to_string()
}

pub(super) fn parse_bounds_from_debug(debug_text: &str) -> (f32, f32, f32, f32) {
    let numbers: Vec<f32> = debug_text
        .split(|character: char| !character.is_numeric() && character != '.' && character != '-')
        .filter_map(|segment| segment.parse::<f32>().ok())
        .collect();

    match numbers.as_slice() {
        [min_x, min_y, max_x, max_y, ..] => (*min_x, *min_y, max_x - min_x, max_y - min_y),
        _ => EMPTY_BOUNDS,
    }
}
