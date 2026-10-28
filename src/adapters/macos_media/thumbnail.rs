use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use objc2::AllocAnyThread;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep};
use objc2_av_foundation::AVAssetImageGenerator;
use objc2_core_foundation::CGSize;
use objc2_core_media::CMTime;
use objc2_foundation::NSDictionary;

use super::asset_reading::open_asset;
use super::foundation_helpers::describe_error;

const THUMBNAIL_MAXIMUM_EDGE: f64 = 640.0;
const THUMBNAIL_TIME_VALUE: i64 = 3;
const THUMBNAIL_TIME_SCALE: i32 = 10;
const PREVIEW_ENCODE_FAILED_ERROR: &str = "could not encode the preview image";
const PREVIEW_SAVE_FAILED_ERROR: &str = "could not save the preview image";

#[allow(deprecated)]
pub fn write_thumbnail(video_file_path: &Path, thumbnail_path: &Path) -> Result<PathBuf> {
    let asset = open_asset(video_file_path);
    let generator = unsafe { AVAssetImageGenerator::assetImageGeneratorWithAsset(&asset) };
    unsafe {
        generator.setAppliesPreferredTrackTransform(true);
        generator.setMaximumSize(CGSize {
            width: THUMBNAIL_MAXIMUM_EDGE,
            height: THUMBNAIL_MAXIMUM_EDGE,
        });
    }
    let requested_time = unsafe { CMTime::new(THUMBNAIL_TIME_VALUE, THUMBNAIL_TIME_SCALE) };
    let image = unsafe {
        generator.copyCGImageAtTime_actualTime_error(requested_time, std::ptr::null_mut())
    }
    .map_err(|error| anyhow!("could not read a preview frame: {}", describe_error(&error)))?;
    let bitmap = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &image);
    let png_data = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }
    .context(PREVIEW_ENCODE_FAILED_ERROR)?;
    std::fs::write(thumbnail_path, png_data.to_vec()).context(PREVIEW_SAVE_FAILED_ERROR)?;
    Ok(thumbnail_path.to_path_buf())
}
