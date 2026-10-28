use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureDevice {
    Microphone,
}

pub trait CapturePermissionGate: Send + Sync {
    fn ensure_access(&self, device: CaptureDevice) -> Result<bool>;
}
