use std::path::Path;

use anyhow::{Context, Result};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Media::MediaFoundation::{
    IMFAttributes, IMFMediaType, MFCreateAttributes, MFCreateMediaType, MFStartup, MFSTARTUP_FULL,
    MF_VERSION,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

pub const HUNDRED_NANOSECONDS_PER_SECOND: i64 = 10_000_000;
const MEDIA_FOUNDATION_UNAVAILABLE_ERROR: &str = "Media Foundation is unavailable";
const CREATE_ATTRIBUTES_FAILED_ERROR: &str = "could not create media attributes";
const ATTRIBUTES_NOT_CREATED_ERROR: &str = "media attributes were not created";
const CREATE_MEDIA_TYPE_FAILED_ERROR: &str = "could not create a media type";

pub fn ensure_media_foundation() -> Result<bool> {
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) }.context(MEDIA_FOUNDATION_UNAVAILABLE_ERROR)?;
    Ok(true)
}

pub fn wide_path(path: &Path) -> HSTRING {
    HSTRING::from(path.as_os_str())
}

pub fn as_pcwstr(value: &HSTRING) -> PCWSTR {
    PCWSTR(value.as_ptr())
}

pub fn pack_ratio(numerator: u32, denominator: u32) -> u64 {
    ((numerator as u64) << 32) | denominator as u64
}

pub fn create_attributes(capacity: u32) -> Result<IMFAttributes> {
    let mut attributes: Option<IMFAttributes> = None;
    unsafe { MFCreateAttributes(&mut attributes, capacity) }
        .context(CREATE_ATTRIBUTES_FAILED_ERROR)?;
    attributes.context(ATTRIBUTES_NOT_CREATED_ERROR)
}

pub fn create_media_type() -> Result<IMFMediaType> {
    unsafe { MFCreateMediaType() }.context(CREATE_MEDIA_TYPE_FAILED_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_ratio_puts_numerator_in_high_bits() {
        assert_eq!(pack_ratio(1920, 1080), (1920u64 << 32) | 1080);
        assert_eq!(pack_ratio(30, 1), (30u64 << 32) | 1);
    }
}
