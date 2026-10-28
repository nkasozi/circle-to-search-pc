use std::sync::mpsc;

use anyhow::{bail, Result};
use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaType};

use super::foundation_helpers::COMPLETION_TIMEOUT;

const CAMERA_ACCESS_DENIED_ERROR: &str = "Camera access is turned off for Circle to Search. Turn it on in System Settings → Privacy & Security → Camera.";
const MICROPHONE_ACCESS_DENIED_ERROR: &str = "Microphone access is turned off for Circle to Search. Turn it on in System Settings → Privacy & Security → Microphone.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureDeviceKind {
    Camera,
    Microphone,
}

pub fn ensure_capture_access(
    media_type: &AVMediaType,
    device_kind: CaptureDeviceKind,
) -> Result<bool> {
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
    match status {
        AVAuthorizationStatus::Authorized => Ok(true),
        AVAuthorizationStatus::NotDetermined => request_access(media_type, device_kind),
        _ => bail!(denied_message(device_kind)),
    }
}

fn request_access(media_type: &AVMediaType, device_kind: CaptureDeviceKind) -> Result<bool> {
    let (sender, receiver) = mpsc::channel::<bool>();
    let completion = RcBlock::new(move |granted: Bool| {
        let _ = sender.send(granted.as_bool());
    });
    unsafe {
        AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &completion)
    };
    let granted = receiver
        .recv_timeout(COMPLETION_TIMEOUT * 6)
        .unwrap_or(false);
    if !granted {
        bail!(denied_message(device_kind));
    }
    Ok(true)
}

fn denied_message(device_kind: CaptureDeviceKind) -> &'static str {
    match device_kind {
        CaptureDeviceKind::Camera => CAMERA_ACCESS_DENIED_ERROR,
        CaptureDeviceKind::Microphone => MICROPHONE_ACCESS_DENIED_ERROR,
    }
}
