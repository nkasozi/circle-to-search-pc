use std::collections::VecDeque;
use std::time::Duration;

use crate::core::interfaces::ports::TimedVideoFrame;

pub fn take_due_frame(
    queue: &mut VecDeque<TimedVideoFrame>,
    position: Duration,
) -> Option<TimedVideoFrame> {
    let mut latest_due: Option<TimedVideoFrame> = None;
    while queue
        .front()
        .is_some_and(|frame| frame.timestamp <= position)
    {
        latest_due = queue.pop_front();
    }
    latest_due
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{PixelLayout, VideoFrame};
    use std::sync::Arc;

    fn frame_at(milliseconds: u64) -> TimedVideoFrame {
        TimedVideoFrame {
            timestamp: Duration::from_millis(milliseconds),
            frame: VideoFrame {
                width: 1,
                height: 1,
                pixel_layout: PixelLayout::Rgba,
                data: Arc::new(vec![0, 0, 0, 255]),
            },
        }
    }

    #[test]
    fn test_returns_latest_frame_at_or_before_position_and_drops_older() {
        let mut queue: VecDeque<TimedVideoFrame> =
            [0, 33, 66, 100].into_iter().map(frame_at).collect();
        let due = take_due_frame(&mut queue, Duration::from_millis(70)).expect("due frame");
        assert_eq!(due.timestamp, Duration::from_millis(66));
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_returns_nothing_before_first_frame() {
        let mut queue: VecDeque<TimedVideoFrame> = [100].into_iter().map(frame_at).collect();
        assert!(take_due_frame(&mut queue, Duration::from_millis(50)).is_none());
        assert_eq!(queue.len(), 1);
    }
}
