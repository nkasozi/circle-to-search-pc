#![cfg_attr(target_os = "macos", allow(dead_code))]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};

use crate::core::interfaces::ports::{ActiveCapture, MediaSink, ScreenFrameSource};
use crate::core::models::{CaptureArea, PixelLayout, ScreenCaptureRequest, VideoFrame};

#[cfg(test)]
mod tests;

const LOG_TAG: &str = "[XCAP_FRAME_SOURCE]";
const DISPLAY_NOT_FOUND_ERROR: &str = "display not found";
const SCREEN_CAPTURE_THREAD_PANICKED_ERROR: &str = "screen capture thread panicked";
const EVENT_XCAP_MONITOR_LOST: &str = "xcap_monitor_lost";
const EVENT_XCAP_CAPTURE_STARTED: &str = "xcap_capture_started";

pub struct XcapScreenFrameSource;

struct XcapCapture {
    running: Arc<AtomicBool>,
    worker: JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhysicalRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn resolve_physical_region(
    request: &ScreenCaptureRequest,
    monitor_width: u32,
    monitor_height: u32,
    scale_factor: f64,
) -> PhysicalRegion {
    let full_region = PhysicalRegion {
        x: 0,
        y: 0,
        width: monitor_width,
        height: monitor_height,
    };
    let CaptureArea::Region(region) = request.capture_area else {
        return full_region;
    };
    let x = ((region.x - request.display_origin_x as f64) * scale_factor).max(0.0) as u32;
    let y = ((region.y - request.display_origin_y as f64) * scale_factor).max(0.0) as u32;
    let width = ((region.width * scale_factor) as u32)
        .min(monitor_width.saturating_sub(x))
        .max(2);
    let height = ((region.height * scale_factor) as u32)
        .min(monitor_height.saturating_sub(y))
        .max(2);
    PhysicalRegion {
        x,
        y,
        width: width - width % 2,
        height: height - height % 2,
    }
}

impl ScreenFrameSource for XcapScreenFrameSource {
    fn start_capture(
        &self,
        request: ScreenCaptureRequest,
        sink: Arc<dyn MediaSink>,
    ) -> Result<Box<dyn ActiveCapture>> {
        let region = measure_capture_region(&request)?;
        let frame_interval = Duration::from_secs_f64(1.0 / request.frames_per_second.max(1) as f64);
        let running = Arc::new(AtomicBool::new(true));
        let capture_loop = CaptureLoop {
            display_point: (request.display_origin_x, request.display_origin_y),
            region,
            frame_interval,
            running: Arc::clone(&running),
            sink,
        };
        let worker = std::thread::spawn(move || {
            let _ = run_capture_loop(capture_loop);
        });
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_XCAP_CAPTURE_STARTED, "width": region.width, "height": region.height})
        );
        Ok(Box::new(XcapCapture { running, worker }))
    }
}

struct CaptureLoop {
    display_point: (i32, i32),
    region: PhysicalRegion,
    frame_interval: Duration,
    running: Arc<AtomicBool>,
    sink: Arc<dyn MediaSink>,
}

fn measure_capture_region(request: &ScreenCaptureRequest) -> Result<PhysicalRegion> {
    let monitor = xcap::Monitor::from_point(request.display_origin_x, request.display_origin_y)
        .context(DISPLAY_NOT_FOUND_ERROR)?;
    let scale_factor = monitor.scale_factor().unwrap_or(1.0) as f64;
    let monitor_width = monitor
        .width()
        .map_err(|error| anyhow!(error.to_string()))?;
    let monitor_height = monitor
        .height()
        .map_err(|error| anyhow!(error.to_string()))?;
    Ok(resolve_physical_region(
        request,
        monitor_width,
        monitor_height,
        scale_factor,
    ))
}

fn run_capture_loop(capture_loop: CaptureLoop) -> bool {
    let (display_x, display_y) = capture_loop.display_point;
    let Ok(monitor) = xcap::Monitor::from_point(display_x, display_y) else {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_XCAP_MONITOR_LOST})
        );
        return false;
    };
    while capture_loop.running.load(Ordering::SeqCst) {
        let started_at = Instant::now();
        let _ = capture_and_forward_frame(&monitor, capture_loop.region, &capture_loop.sink);
        std::thread::sleep(
            capture_loop
                .frame_interval
                .saturating_sub(started_at.elapsed()),
        );
    }
    true
}

fn capture_and_forward_frame(
    monitor: &xcap::Monitor,
    region: PhysicalRegion,
    sink: &Arc<dyn MediaSink>,
) -> bool {
    let Ok(image) = monitor.capture_region(region.x, region.y, region.width, region.height) else {
        return false;
    };
    let (width, height) = (
        image.width() - image.width() % 2,
        image.height() - image.height() % 2,
    );
    let cropped = image::imageops::crop_imm(&image, 0, 0, width, height).to_image();
    sink.accept_video_frame(VideoFrame {
        width,
        height,
        pixel_layout: PixelLayout::Rgba,
        data: Arc::new(cropped.into_raw()),
    })
}

impl ActiveCapture for XcapCapture {
    fn stop(self: Box<Self>) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.worker
            .join()
            .map_err(|_| anyhow!(SCREEN_CAPTURE_THREAD_PANICKED_ERROR))
    }
}
