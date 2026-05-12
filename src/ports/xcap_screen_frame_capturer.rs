use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{self, Receiver};

use crate::core::interfaces::ports::ScreenFrameCapturer;
use crate::core::models::recording::{RecordingConfig, RecordingFrame};
use crate::core::models::ScreenRegion;

const LOG_TAG: &str = "[SCREEN_FRAME_CAPTURER]";
const FRAME_CHANNEL_BUFFER_SIZE: usize = 5;
const ERROR_NO_MONITOR_AT_POSITION: &str = "failed to find monitor at capture position";
const ERROR_FAILED_TO_CAPTURE_FRAME: &str = "failed to capture screen frame";
const ERROR_SCALE_FACTOR_UNAVAILABLE: &str = "unable to get scale factor from monitor";

pub struct XcapScreenFrameCapturer {
    is_capturing: Arc<AtomicBool>,
}

impl XcapScreenFrameCapturer {
    pub fn initialize() -> Self {
        log::debug!("{} initializing xcap screen frame capturer", LOG_TAG);
        Self {
            is_capturing: Arc::new(AtomicBool::new(false)),
        }
    }

    fn find_monitor_at_region(region: &ScreenRegion) -> Result<xcap::Monitor> {
        xcap::Monitor::from_point(region.x_position, region.y_position)
            .with_context(|| ERROR_NO_MONITOR_AT_POSITION)
    }

    fn extract_scale_factor(monitor: &xcap::Monitor) -> Result<f64> {
        let scale_factor = monitor
            .scale_factor()
            .with_context(|| ERROR_SCALE_FACTOR_UNAVAILABLE)?;

        Ok(scale_factor as f64)
    }

    fn capture_single_frame(
        monitor: &xcap::Monitor,
        recording_start_time: &Instant,
    ) -> Result<RecordingFrame> {
        let captured_image = monitor
            .capture_image()
            .with_context(|| ERROR_FAILED_TO_CAPTURE_FRAME)?;

        let timestamp = recording_start_time.elapsed();
        let width_pixels = captured_image.width();
        let height_pixels = captured_image.height();
        let raw_rgba_data = captured_image.into_raw();

        Ok(RecordingFrame::build(
            timestamp,
            width_pixels,
            height_pixels,
            raw_rgba_data,
        ))
    }
}

impl ScreenFrameCapturer for XcapScreenFrameCapturer {
    fn start_continuous_capture(
        &self,
        config: &RecordingConfig,
    ) -> Result<Receiver<RecordingFrame>> {
        log::info!(
            "{} starting continuous capture at {} fps for region ({}, {})",
            LOG_TAG,
            config.frames_per_second,
            config.capture_region_x,
            config.capture_region_y,
        );

        let region = ScreenRegion::at_coordinates(config.capture_region_x, config.capture_region_y);

        let monitor = Self::find_monitor_at_region(&region)?;
        let _scale_factor = Self::extract_scale_factor(&monitor)?;

        self.is_capturing.store(true, Ordering::SeqCst);

        let (frame_sender, frame_receiver) =
            mpsc::channel::<RecordingFrame>(FRAME_CHANNEL_BUFFER_SIZE);

        let is_capturing_flag = self.is_capturing.clone();
        let frame_interval_millis = 1000 / config.frames_per_second.max(1) as u64;
        let capture_region_x = config.capture_region_x;
        let capture_region_y = config.capture_region_y;

        tokio::spawn(async move {
            let recording_start_time = Instant::now();
            let mut frame_interval =
                tokio::time::interval(std::time::Duration::from_millis(frame_interval_millis));

            log::info!(
                "{} capture loop started, interval={}ms",
                LOG_TAG,
                frame_interval_millis,
            );

            loop {
                frame_interval.tick().await;

                if !is_capturing_flag.load(Ordering::SeqCst) {
                    log::info!("{} capture stopped by flag", LOG_TAG);
                    break;
                }

                let region = ScreenRegion::at_coordinates(capture_region_x, capture_region_y);

                let monitor = match Self::find_monitor_at_region(&region) {
                    Ok(found_monitor) => found_monitor,
                    Err(monitor_error) => {
                        log::error!(
                            "{} monitor lookup failed during capture: {}",
                            LOG_TAG,
                            monitor_error,
                        );
                        break;
                    }
                };

                match Self::capture_single_frame(&monitor, &recording_start_time) {
                    Ok(frame) => {
                        if frame_sender.send(frame).await.is_err() {
                            log::info!("{} frame receiver dropped, stopping capture", LOG_TAG);
                            break;
                        }
                    }
                    Err(capture_error) => {
                        log::warn!(
                            "{} frame capture failed, skipping: {}",
                            LOG_TAG,
                            capture_error,
                        );
                    }
                }
            }

            is_capturing_flag.store(false, Ordering::SeqCst);
            log::info!("{} capture loop ended", LOG_TAG);
        });

        Ok(frame_receiver)
    }

    fn stop_continuous_capture(&self) -> Result<bool> {
        log::info!("{} stopping continuous capture", LOG_TAG);
        self.is_capturing.store(false, Ordering::SeqCst);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_creates_instance_with_capturing_false() {
        let capturer = XcapScreenFrameCapturer::initialize();
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stop_continuous_capture_sets_flag_to_false() {
        let capturer = XcapScreenFrameCapturer::initialize();
        capturer.is_capturing.store(true, Ordering::SeqCst);

        let result = capturer.stop_continuous_capture();

        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stop_continuous_capture_is_idempotent() {
        let capturer = XcapScreenFrameCapturer::initialize();

        let first_result = capturer.stop_continuous_capture();
        let second_result = capturer.stop_continuous_capture();

        assert!(first_result.is_ok());
        assert!(second_result.is_ok());
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }
}
