#[derive(Debug)]
pub struct ScreenRegion {
    pub x_position: i32,
    pub y_position: i32,
}

impl ScreenRegion {
    pub fn at_coordinates(x_position: i32, y_position: i32) -> Self {
        log::debug!(
            "[SCREEN_REGION] creating region at ({}, {})",
            x_position,
            y_position
        );

        Self {
            x_position,
            y_position,
        }
    }

    #[allow(dead_code)]
    pub fn default_origin() -> Self {
        log::debug!("[SCREEN_REGION] using default origin (0, 0)");

        Self {
            x_position: 0,
            y_position: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_coordinates_creates_region_with_correct_position() {
        let region = ScreenRegion::at_coordinates(100, 200);

        assert_eq!(region.x_position, 100);
        assert_eq!(region.y_position, 200);
    }

    #[test]
    fn test_at_coordinates_handles_negative_coordinates() {
        let region = ScreenRegion::at_coordinates(-50, -100);

        assert_eq!(region.x_position, -50);
        assert_eq!(region.y_position, -100);
    }

    #[test]
    fn test_default_origin_creates_region_at_zero() {
        let region = ScreenRegion::default_origin();

        assert_eq!(region.x_position, 0);
        assert_eq!(region.y_position, 0);
    }
}
