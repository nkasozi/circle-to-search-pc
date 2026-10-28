use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerState {
    pub x: f64,
    pub y: f64,
    pub is_primary_button_pressed: bool,
}

pub trait PointerStateProvider: Send + Sync {
    fn current_pointer_state(&self) -> Result<PointerState>;
}
