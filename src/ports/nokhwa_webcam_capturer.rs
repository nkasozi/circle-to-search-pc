use anyhow::{Context, Result};
use nokhwa::pixel_format::RgbAFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::Camera;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{self, Receiver};

use crate::core::interfaces::ports::WebcamCapturer;
use crate::core::models::recording::RecordingFrame;

const LOG_TAG: &str = "[WEBCAM_CAPTURER]";
const WEBCAM_CHANNEL_BUFFER_SIZE: usize = 5;
const WEBCAM_FRAME_INTERVAL_MILLIS: u64 = 33;
const DEFAULT_CAMERA_INDEX: u32 = 0;
const DEFAULT_WEBCAM_FRAME_RATE: u32 = 30;
const ERROR_FAILED_TO_OPEN_CAMERA: &str = "failed to open webcam camera";
const ERROR_FAILED_TO_CAPTURE_FRAME: &str = "failed to capture webcam frame";

pub struct NokhwaWebcamCapturer {
    is_capturing: Arc<AtomicBool>,
}

impl NokhwaWebcamCapturer {
    pub fn initialize() -> Self {
        log::debug!("{} initializing nokhwa webcam capturer", LOG_TAG);
        Self {
            is_capturing: Arc::new(AtomicBool::new(false)),
        }
    }

    fn open_camera(requested_width: u32, requested_height: u32) -> Result<Camera> {
        let camera_index = CameraIndex::Index(DEFAULT_CAMERA_INDEX);
        let requested_resolution = Resolution::new(requested_width, requested_height);
        let camera_format = CameraFormat::new(
            requested_resolution,
            FrameFormat::MJPEG,
            DEFAULT_WEBCAM_FRAME_RATE,
        );

        let requested_format =
            RequestedFormat::new::<RgbAFormat>(RequestedFormatType::Closest(camera_format));

        log::info!(
            "{} opening camera index={}, requested={}x{}",
            LOG_TAG,
            DEFAULT_CAMERA_INDEX,
            requested_width,
            requested_height,
        );

        Camera::new(camera_index, requested_format).with_context(|| ERROR_FAILED_TO_OPEN_CAMERA)
    }

    fn capture_frame_from_camera(
        camera: &mut Camera,
        recording_start_time: &Instant,
    ) -> Result<RecordingFrame> {
        let frame_buffer = camera
            .frame()
            .with_context(|| ERROR_FAILED_TO_CAPTURE_FRAME)?;

        let decoded = frame_buffer
            .decode_image::<RgbAFormat>()
            .with_context(|| ERROR_FAILED_TO_CAPTURE_FRAME)?;

        let width_pixels = decoded.width();
        let height_pixels = decoded.height();
        let raw_rgba_data = decoded.into_raw();
        let timestamp = recording_start_time.elapsed();

        Ok(RecordingFrame::build(
            timestamp,
            width_pixels,
            height_pixels,
            raw_rgba_data,
        ))
    }

    fn run_capture_loop(
        requested_width: u32,
        requested_height: u32,
        is_capturing_flag: Arc<AtomicBool>,
        frame_sender: mpsc::Sender<RecordingFrame>,
    ) {
        let camera_result = Self::open_camera(requested_width, requested_height);
        let mut camera = match camera_result {
            Ok(opened_camera) => opened_camera,
            Err(open_error) => {
                log::error!(
                    "{} failed to open camera in capture thread: {}",
                    LOG_TAG,
                    open_error,
                );
                is_capturing_flag.store(false, Ordering::SeqCst);
                return;
            }
        };

        if let Err(stream_error) = camera.open_stream() {
            log::error!("{} failed to open camera stream: {}", LOG_TAG, stream_error,);
            is_capturing_flag.store(false, Ordering::SeqCst);
            return;
        }

        let actual_resolution = camera.resolution();
        log::info!(
            "{} webcam opened in thread, actual resolution={}x{}",
            LOG_TAG,
            actual_resolution.width(),
            actual_resolution.height(),
        );

        let recording_start_time = Instant::now();
        log::info!("{} webcam capture loop started", LOG_TAG);

        loop {
            if !is_capturing_flag.load(Ordering::SeqCst) {
                log::info!("{} webcam capture stopped by flag", LOG_TAG);
                break;
            }

            match Self::capture_frame_from_camera(&mut camera, &recording_start_time) {
                Ok(frame) => {
                    if frame_sender.blocking_send(frame).is_err() {
                        log::info!("{} webcam frame receiver dropped, stopping", LOG_TAG,);
                        break;
                    }
                }
                Err(capture_error) => {
                    log::warn!(
                        "{} webcam frame capture failed, skipping: {}",
                        LOG_TAG,
                        capture_error,
                    );
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(
                WEBCAM_FRAME_INTERVAL_MILLIS,
            ));
        }

        is_capturing_flag.store(false, Ordering::SeqCst);
        log::info!("{} webcam capture loop ended", LOG_TAG);
    }
}

impl WebcamCapturer for NokhwaWebcamCapturer {
    fn start_webcam_capture(
        &self,
        requested_width: u32,
        requested_height: u32,
    ) -> Result<Receiver<RecordingFrame>> {
        log::info!(
            "{} starting webcam capture at {}x{}",
            LOG_TAG,
            requested_width,
            requested_height,
        );

        self.is_capturing.store(true, Ordering::SeqCst);

        let (frame_sender, frame_receiver) =
            mpsc::channel::<RecordingFrame>(WEBCAM_CHANNEL_BUFFER_SIZE);

        let is_capturing_flag = self.is_capturing.clone();

        std::thread::spawn(move || {
            Self::run_capture_loop(
                requested_width,
                requested_height,
                is_capturing_flag,
                frame_sender,
            );
        });

        Ok(frame_receiver)
    }

    fn stop_webcam_capture(&self) -> Result<bool> {
        log::info!("{} stopping webcam capture", LOG_TAG);
        self.is_capturing.store(false, Ordering::SeqCst);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_creates_instance_with_capturing_false() {
        let capturer = NokhwaWebcamCapturer::initialize();
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stop_webcam_capture_sets_flag_to_false() {
        let capturer = NokhwaWebcamCapturer::initialize();
        capturer.is_capturing.store(true, Ordering::SeqCst);

        let result = capturer.stop_webcam_capture();

        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stop_webcam_capture_is_idempotent() {
        let capturer = NokhwaWebcamCapturer::initialize();

        let first_result = capturer.stop_webcam_capture();
        let second_result = capturer.stop_webcam_capture();

        assert!(first_result.is_ok());
        assert!(second_result.is_ok());
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }
}
