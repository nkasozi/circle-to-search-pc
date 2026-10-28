use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

use crate::adapters::camera_frame_processing::{
    build_mirrored_square_preview, CameraImageGeometry, SourceChannelOrder,
};
use crate::core::interfaces::ports::{ActiveCapture, CameraSource, MediaSink};

const LOG_TAG: &str = "[CAMERA]";
const CAMERA_PREVIEW_EDGE: usize = 480;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_CAMERA_INDEX: u32 = 0;
const RGB_BYTES_PER_PIXEL: usize = 3;
const NO_CAMERA_ERROR: &str = "No camera was found";
const CAMERA_START_FAILED_ERROR: &str = "could not start the camera";
const CAMERA_START_TIMEOUT_ERROR: &str = "camera did not start in time";
const CAMERA_THREAD_PANICKED_ERROR: &str = "camera thread panicked";
const EVENT_CAMERA_CAPTURE_STARTED: &str = "camera_capture_started";

pub struct NokhwaCameraSource;

struct NokhwaCapture {
    running: Arc<AtomicBool>,
    worker: JoinHandle<()>,
}

fn open_camera() -> Result<Camera> {
    let requested_format =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(CameraIndex::Index(DEFAULT_CAMERA_INDEX), requested_format)
        .context(NO_CAMERA_ERROR)?;
    camera.open_stream().context(CAMERA_START_FAILED_ERROR)?;
    Ok(camera)
}

fn forward_next_frame(camera: &mut Camera, sink: &Arc<dyn MediaSink>) -> Result<bool> {
    let decoded = camera.frame()?.decode_image::<RgbFormat>()?;
    let geometry = CameraImageGeometry {
        width: decoded.width() as usize,
        height: decoded.height() as usize,
        bytes_per_row: decoded.width() as usize * RGB_BYTES_PER_PIXEL,
        channel_order: SourceChannelOrder::Rgb,
    };
    let preview = build_mirrored_square_preview(decoded.as_raw(), geometry, CAMERA_PREVIEW_EDGE);
    Ok(preview
        .map(|frame| sink.accept_video_frame(frame))
        .unwrap_or(false))
}

impl CameraSource for NokhwaCameraSource {
    fn start_capture(&self, sink: Arc<dyn MediaSink>) -> Result<Box<dyn ActiveCapture>> {
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let (startup_sender, startup_receiver) = mpsc::channel::<std::result::Result<(), String>>();
        let worker = std::thread::spawn(move || {
            let mut camera = match open_camera() {
                Ok(camera) => camera,
                Err(open_error) => {
                    let _ = startup_sender.send(Err(open_error.to_string()));
                    return;
                }
            };
            let _ = startup_sender.send(Ok(()));
            while worker_running.load(Ordering::SeqCst) {
                let _ = forward_next_frame(&mut camera, &sink);
            }
            let _ = camera.stop_stream();
        });
        startup_receiver
            .recv_timeout(STARTUP_TIMEOUT)
            .context(CAMERA_START_TIMEOUT_ERROR)?
            .map_err(|message| anyhow!(message))?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_CAMERA_CAPTURE_STARTED})
        );
        Ok(Box::new(NokhwaCapture { running, worker }))
    }
}

impl ActiveCapture for NokhwaCapture {
    fn stop(self: Box<Self>) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.worker
            .join()
            .map_err(|_| anyhow!(CAMERA_THREAD_PANICKED_ERROR))
    }
}
