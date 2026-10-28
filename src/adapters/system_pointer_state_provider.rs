use anyhow::Result;

use crate::core::interfaces::ports::{PointerState, PointerStateProvider};

pub struct SystemPointerStateProvider;

#[cfg(target_os = "macos")]
impl PointerStateProvider for SystemPointerStateProvider {
    fn current_pointer_state(&self) -> Result<PointerState> {
        use objc2_app_kit::NSEvent;
        use objc2_core_graphics::{CGDisplayBounds, CGMainDisplayID};
        const PRIMARY_BUTTON_MASK: usize = 1;
        let location = NSEvent::mouseLocation();
        let primary_display_height = CGDisplayBounds(CGMainDisplayID()).size.height;
        Ok(PointerState {
            x: location.x,
            y: primary_display_height - location.y,
            is_primary_button_pressed: NSEvent::pressedMouseButtons() & PRIMARY_BUTTON_MASK != 0,
        })
    }
}

#[cfg(not(target_os = "macos"))]
impl PointerStateProvider for SystemPointerStateProvider {
    fn current_pointer_state(&self) -> Result<PointerState> {
        use device_query::{DeviceQuery, DeviceState};
        const PRIMARY_BUTTON_INDEX: usize = 1;
        let mouse = DeviceState::new().get_mouse();
        let (physical_x, physical_y) = mouse.coords;
        let scale_factor = xcap::Monitor::from_point(physical_x, physical_y)
            .and_then(|monitor| monitor.scale_factor())
            .unwrap_or(1.0) as f64;
        Ok(PointerState {
            x: physical_x as f64 / scale_factor,
            y: physical_y as f64 / scale_factor,
            is_primary_button_pressed: mouse
                .button_pressed
                .get(PRIMARY_BUTTON_INDEX)
                .copied()
                .unwrap_or(false),
        })
    }
}
