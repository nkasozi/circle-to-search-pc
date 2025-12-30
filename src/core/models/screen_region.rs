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
