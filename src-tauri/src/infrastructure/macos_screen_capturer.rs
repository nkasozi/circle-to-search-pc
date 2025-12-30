use crate::core::interfaces::ScreenCapturePort;
use crate::core::models::{Display, ScreenCapture, ImageFormat};
use std::process::Command;

pub struct MacOSScreenCapturer;

impl ScreenCapturePort for MacOSScreenCapturer {
    fn capture_display(&self, display: &Display) -> Result<ScreenCapture, String> {
        let temp_path = "/tmp/screenshot_circle_search.png";

        let output = Command::new("screencapture")
            .arg("-x")
            .arg("-S")
            .arg(temp_path)
            .output()
            .map_err(|e| format!("Failed to run screencapture: {}", e))?;

        if !output.status.success() {
            return Err(format!("screencapture failed: {}",
                String::from_utf8_lossy(&output.stderr)));
        }

        let png_data = std::fs::read(temp_path)
            .map_err(|e| format!("Failed to read screenshot file: {}", e))?;

        let _ = std::fs::remove_file(temp_path);

        Ok(ScreenCapture {
            image_data: png_data,
            width: display.width,
            height: display.height,
            format: ImageFormat::PNG,
        })
    }

    fn get_all_displays(&self) -> Result<Vec<Display>, String> {
        Ok(vec![Display {
            id: 0,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        }])
    }
}
