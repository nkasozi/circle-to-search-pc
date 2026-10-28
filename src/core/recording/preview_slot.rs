use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::core::interfaces::ports::MediaSink;
use crate::core::models::{AudioChunk, AudioTrackKind, VideoFrame};

#[derive(Default)]
pub struct PreviewFrameSlot {
    latest_frame: Mutex<Option<VideoFrame>>,
    sequence: AtomicU64,
}

impl PreviewFrameSlot {
    pub fn take_newer_than(&self, last_seen_sequence: u64) -> Option<(u64, VideoFrame)> {
        let current_sequence = self.sequence.load(Ordering::SeqCst);
        if current_sequence == last_seen_sequence {
            return None;
        }
        let frame = self.latest_frame.lock().ok()?.clone()?;
        Some((current_sequence, frame))
    }
}

impl MediaSink for PreviewFrameSlot {
    fn accept_video_frame(&self, frame: VideoFrame) -> bool {
        let stored = self
            .latest_frame
            .lock()
            .map(|mut slot| slot.replace(frame).is_some())
            .unwrap_or(false);
        self.sequence.fetch_add(1, Ordering::SeqCst);
        stored
    }

    fn accept_audio_chunk(&self, _track: AudioTrackKind, _chunk: AudioChunk) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::PixelLayout;
    use std::sync::Arc;

    fn frame() -> VideoFrame {
        VideoFrame {
            width: 1,
            height: 1,
            pixel_layout: PixelLayout::Rgba,
            data: Arc::new(vec![0, 0, 0, 255]),
        }
    }

    #[test]
    fn test_only_newer_frames_are_returned() {
        let slot = PreviewFrameSlot::default();
        assert!(slot.take_newer_than(0).is_none());
        slot.accept_video_frame(frame());
        let (sequence, _) = slot.take_newer_than(0).expect("new frame");
        assert!(slot.take_newer_than(sequence).is_none());
    }
}
