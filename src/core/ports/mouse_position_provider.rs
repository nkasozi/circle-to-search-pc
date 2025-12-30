use crate::core::models::ScreenRegion;

pub trait MousePositionProvider: Send + Sync {
    #[allow(dead_code)]
    fn get_current_mouse_position(&self) -> Result<ScreenRegion, String>;
}
