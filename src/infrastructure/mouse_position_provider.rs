use crate::core::models::ScreenRegion;
use crate::core::ports::MousePositionProvider;
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
