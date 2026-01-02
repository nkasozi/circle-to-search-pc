use crate::core::interfaces::ports::MousePositionProvider;
use crate::core::models::ScreenRegion;
use crate::global_constants::MESSAGE_MOUSE_POSITION_FAILED;
use mouse_position::mouse_position::Mouse;

pub struct SystemMousePositionProvider;

impl SystemMousePositionProvider {
    pub fn initialize() -> Self {
        log::debug!("[MOUSE] initializing mouse position provider");
        Self
    }

    #[allow(dead_code)]
    fn query_system_mouse_position(&self) -> Mouse {
        Mouse::get_mouse_position()
    }

    #[allow(dead_code)]
    fn convert_mouse_result_to_region(&self, mouse_result: Mouse) -> Result<ScreenRegion, String> {
        match mouse_result {
            Mouse::Position { x, y } => {
                log::debug!("[MOUSE] current position: ({}, {})", x, y);
                Ok(ScreenRegion::at_coordinates(x, y))
            }
            Mouse::Error => {
                log::warn!("[MOUSE] {}", MESSAGE_MOUSE_POSITION_FAILED);
                Err(MESSAGE_MOUSE_POSITION_FAILED.to_string())
            }
        }
    }
}

impl MousePositionProvider for SystemMousePositionProvider {
    fn get_current_mouse_position(&self) -> Result<ScreenRegion, String> {
        let mouse_position_result = self.query_system_mouse_position();
        self.convert_mouse_result_to_region(mouse_position_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_creates_provider() {
        let provider = SystemMousePositionProvider::initialize();

        assert!(std::mem::size_of_val(&provider) == 0);
    }

    #[test]
    fn test_convert_mouse_result_to_region_with_valid_position_returns_ok() {
        let provider = SystemMousePositionProvider::initialize();
        let mouse_result = Mouse::Position { x: 100, y: 200 };

        let result = provider.convert_mouse_result_to_region(mouse_result);

        assert!(result.is_ok());
        let region = result.unwrap();
        assert_eq!(region.x_position, 100);
        assert_eq!(region.y_position, 200);
    }

    #[test]
    fn test_convert_mouse_result_to_region_with_error_returns_err() {
        let provider = SystemMousePositionProvider::initialize();
        let mouse_result = Mouse::Error;

        let result = provider.convert_mouse_result_to_region(mouse_result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mouse position"));
    }

    #[test]
    fn test_convert_mouse_result_to_region_handles_negative_coordinates() {
        let provider = SystemMousePositionProvider::initialize();
        let mouse_result = Mouse::Position { x: -50, y: -100 };

        let result = provider.convert_mouse_result_to_region(mouse_result);

        assert!(result.is_ok());
        let region = result.unwrap();
        assert_eq!(region.x_position, -50);
        assert_eq!(region.y_position, -100);
    }
}
