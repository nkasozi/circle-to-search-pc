use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::image;
use iced::Task;

use super::RecordingMessage;
use crate::core::interfaces::ports::VideoEncoder;

const THUMBNAIL_FILE_SUFFIX: &str = ".thumbnail.png";

fn thumbnail_message(generated: Option<(PathBuf, PathBuf)>) -> RecordingMessage {
    match generated {
        Some((video_path, thumbnail_path)) => RecordingMessage::ThumbnailReady(
            video_path,
            Some(image::Handle::from_path(thumbnail_path)),
        ),
        None => RecordingMessage::ThumbnailReady(PathBuf::new(), None),
    }
}

pub fn build_thumbnail_task(
    encoder: Arc<dyn VideoEncoder>,
    video_path: PathBuf,
) -> Task<RecordingMessage> {
    let thumbnail_path =
        std::env::temp_dir().join(format!("{}{}", uuid::Uuid::new_v4(), THUMBNAIL_FILE_SUFFIX));
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                encoder
                    .extract_thumbnail(&video_path, &thumbnail_path)
                    .ok()
                    .map(|path| (video_path, path))
            })
            .await
            .ok()
            .flatten()
        },
        thumbnail_message,
    )
}
