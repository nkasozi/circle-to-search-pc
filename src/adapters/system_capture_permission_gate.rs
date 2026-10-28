use anyhow::Result;

use crate::core::interfaces::ports::{CaptureDevice, CapturePermissionGate};

#[cfg(target_os = "macos")]
const MEDIA_TYPE_UNAVAILABLE_ERROR: &str = "media type unavailable";

pub struct SystemCapturePermissionGate;

#[cfg(target_os = "macos")]
impl CapturePermissionGate for SystemCapturePermissionGate {
    fn ensure_access(&self, device: CaptureDevice) -> Result<bool> {
        use crate::adapters::macos_media::{ensure_capture_access, CaptureDeviceKind};
        use anyhow::Context;
        let (media_type, device_kind) = match device {
            CaptureDevice::Microphone => (
                unsafe { objc2_av_foundation::AVMediaTypeAudio },
                CaptureDeviceKind::Microphone,
            ),
        };
        ensure_capture_access(
            media_type.context(MEDIA_TYPE_UNAVAILABLE_ERROR)?,
            device_kind,
        )
    }
}

#[cfg(not(target_os = "macos"))]
impl CapturePermissionGate for SystemCapturePermissionGate {
    fn ensure_access(&self, _device: CaptureDevice) -> Result<bool> {
        Ok(true)
    }
}
