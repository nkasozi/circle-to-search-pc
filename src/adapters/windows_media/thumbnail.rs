use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::frame_reader::VideoFrameReader;

const THUMBNAIL_MAXIMUM_WIDTH: u32 = 640;
const NO_VIDEO_FRAMES_ERROR: &str = "the recording has no video frames";
const INVALID_VIDEO_FRAME_ERROR: &str = "invalid video frame";
const PREVIEW_SAVE_FAILED_ERROR: &str = "could not save the preview image";

pub fn write_thumbnail(video_file_path: &Path, thumbnail_path: &Path) -> Result<PathBuf> {
    let mut reader = VideoFrameReader::open(video_file_path)?;
    let decoded = reader.read_next()?.context(NO_VIDEO_FRAMES_ERROR)?;
    let frame = decoded.frame;
    let image = image::RgbaImage::from_raw(frame.width, frame.height, frame.data.to_vec())
        .context(INVALID_VIDEO_FRAME_ERROR)?;
    let scale = (THUMBNAIL_MAXIMUM_WIDTH as f32 / frame.width as f32).min(1.0);
    let thumbnail = image::imageops::thumbnail(
        &image,
        (frame.width as f32 * scale) as u32,
        (frame.height as f32 * scale) as u32,
    );
    thumbnail
        .save(thumbnail_path)
        .context(PREVIEW_SAVE_FAILED_ERROR)?;
    Ok(thumbnail_path.to_path_buf())
}
