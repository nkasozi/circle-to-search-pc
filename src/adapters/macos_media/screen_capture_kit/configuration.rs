use objc2::rc::Retained;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_media::CMTime;
use objc2_core_video::kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange;
use objc2_screen_capture_kit::{SCDisplay, SCStreamConfiguration};

use crate::core::models::{CaptureArea, ScreenCaptureRequest};

const STREAM_QUEUE_DEPTH: isize = 6;
const SYSTEM_AUDIO_SAMPLE_RATE: isize = 48_000;
const SYSTEM_AUDIO_CHANNEL_COUNT: isize = 2;

pub fn build_stream_configuration(
    request: &ScreenCaptureRequest,
    display: &SCDisplay,
) -> Retained<SCStreamConfiguration> {
    let configuration = unsafe { SCStreamConfiguration::new() };
    unsafe {
        configuration.setWidth(request.output.width as usize);
        configuration.setHeight(request.output.height as usize);
        configuration
            .setMinimumFrameInterval(CMTime::new(1, request.frames_per_second.max(1) as i32));
        configuration.setPixelFormat(kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange);
        configuration.setShowsCursor(request.shows_cursor);
        configuration.setQueueDepth(STREAM_QUEUE_DEPTH);
        configuration.setCapturesAudio(request.captures_system_audio);
        configuration.setSampleRate(SYSTEM_AUDIO_SAMPLE_RATE);
        configuration.setChannelCount(SYSTEM_AUDIO_CHANNEL_COUNT);
        configuration.setExcludesCurrentProcessAudio(true);
    }
    if let CaptureArea::Region(region) = request.capture_area {
        let display_frame = unsafe { display.frame() };
        let source_rect = CGRect {
            origin: CGPoint {
                x: region.x - display_frame.origin.x,
                y: region.y - display_frame.origin.y,
            },
            size: CGSize {
                width: region.width,
                height: region.height,
            },
        };
        unsafe { configuration.setSourceRect(source_rect) };
    }
    configuration
}
