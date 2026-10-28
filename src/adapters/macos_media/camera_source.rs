use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use dispatch2::{DispatchQueue, DispatchQueueAttr, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass};
use objc2_av_foundation::{
    AVCaptureConnection, AVCaptureDevice, AVCaptureDeviceInput, AVCaptureOutput, AVCaptureSession,
    AVCaptureSessionPreset1280x720, AVCaptureVideoDataOutput,
    AVCaptureVideoDataOutputSampleBufferDelegate, AVMediaTypeVideo,
};
use objc2_core_media::CMSampleBuffer;
use objc2_core_video::{
    kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_32BGRA, CVPixelBuffer,
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags,
    CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{NSObject, NSObjectProtocol};

use super::camera_permission::{ensure_capture_access, CaptureDeviceKind};
use super::foundation_helpers::{
    bridged_key, build_dictionary, describe_error, number_object, required_static,
};
use super::system_constant_names::AV_MEDIA_TYPE_VIDEO_NAME;
use crate::adapters::camera_frame_processing::{
    build_mirrored_square_preview, CameraImageGeometry, SourceChannelOrder,
};
use crate::core::interfaces::ports::{ActiveCapture, CameraSource, MediaSink};

const LOG_TAG: &str = "[CAMERA]";
const NO_CAMERA_ERROR: &str = "No camera was found";
const CAMERA_QUEUE_LABEL: &str = "circle-to-search.camera";
const CAMERA_BUSY_ERROR: &str = "the camera is busy or unavailable";
const CAMERA_CAPTURE_STARTED_EVENT: &str = "camera_capture_started";
const CAMERA_CAPTURE_STOPPED_EVENT: &str = "camera_capture_stopped";
pub const CAMERA_PREVIEW_EDGE: usize = 480;

pub struct AvFoundationCameraSource;

pub struct CameraOutputState {
    sink: Arc<dyn MediaSink>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "CircleToSearchCameraOutput"]
    #[ivars = CameraOutputState]
    pub struct CameraOutputHandler;

    unsafe impl NSObjectProtocol for CameraOutputHandler {}

    unsafe impl AVCaptureVideoDataOutputSampleBufferDelegate for CameraOutputHandler {
        #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
        unsafe fn capture_output_did_output(
            &self,
            _output: &AVCaptureOutput,
            sample_buffer: &CMSampleBuffer,
            _connection: &AVCaptureConnection,
        ) {
            let _ = self.forward_frame(sample_buffer);
        }
    }
);

impl CameraOutputHandler {
    fn build(sink: Arc<dyn MediaSink>) -> Retained<Self> {
        let allocated = Self::alloc().set_ivars(CameraOutputState { sink });
        unsafe { msg_send![super(allocated), init] }
    }

    fn forward_frame(&self, sample_buffer: &CMSampleBuffer) -> bool {
        let Some(image_buffer) = (unsafe { sample_buffer.image_buffer() }) else {
            return false;
        };
        let pixel_buffer: &CVPixelBuffer = &image_buffer;
        unsafe { CVPixelBufferLockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
        let preview = build_preview(pixel_buffer);
        unsafe { CVPixelBufferUnlockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
        preview
            .map(|frame| self.ivars().sink.accept_video_frame(frame))
            .unwrap_or(false)
    }
}

fn build_preview(pixel_buffer: &CVPixelBuffer) -> Option<crate::core::models::VideoFrame> {
    let base_address = CVPixelBufferGetBaseAddress(pixel_buffer) as *const u8;
    if base_address.is_null() {
        return None;
    }
    let geometry = CameraImageGeometry {
        width: CVPixelBufferGetWidth(pixel_buffer),
        height: CVPixelBufferGetHeight(pixel_buffer),
        bytes_per_row: CVPixelBufferGetBytesPerRow(pixel_buffer),
        channel_order: SourceChannelOrder::Bgra,
    };
    let source = unsafe {
        std::slice::from_raw_parts(base_address, geometry.bytes_per_row * geometry.height)
    };
    build_mirrored_square_preview(source, geometry, CAMERA_PREVIEW_EDGE)
}

struct AvFoundationCameraCapture {
    session: Retained<AVCaptureSession>,
    _handler: Retained<CameraOutputHandler>,
    _queue: DispatchRetained<DispatchQueue>,
}

unsafe impl Send for AvFoundationCameraCapture {}

impl CameraSource for AvFoundationCameraSource {
    fn start_capture(&self, sink: Arc<dyn MediaSink>) -> Result<Box<dyn ActiveCapture>> {
        let video_type = required_static(unsafe { AVMediaTypeVideo }, AV_MEDIA_TYPE_VIDEO_NAME)?;
        ensure_capture_access(video_type, CaptureDeviceKind::Camera)?;
        let device = unsafe { AVCaptureDevice::defaultDeviceWithMediaType(video_type) }
            .context(NO_CAMERA_ERROR)?;
        let input = unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }
            .map_err(|error| anyhow!("could not open the camera: {}", describe_error(&error)))?;
        let session = unsafe { AVCaptureSession::new() };
        let output = unsafe { AVCaptureVideoDataOutput::new() };
        let handler = CameraOutputHandler::build(sink);
        let queue = DispatchQueue::new(CAMERA_QUEUE_LABEL, DispatchQueueAttr::SERIAL);
        configure_session(&session, &input, &output, &handler, &queue)?;
        unsafe { session.startRunning() };
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CAMERA_CAPTURE_STARTED_EVENT})
        );
        Ok(Box::new(AvFoundationCameraCapture {
            session,
            _handler: handler,
            _queue: queue,
        }))
    }
}

fn configure_session(
    session: &AVCaptureSession,
    input: &AVCaptureDeviceInput,
    output: &AVCaptureVideoDataOutput,
    handler: &CameraOutputHandler,
    queue: &DispatchQueue,
) -> Result<()> {
    let video_settings = build_dictionary(&[(
        bridged_key(unsafe { kCVPixelBufferPixelFormatTypeKey }),
        number_object(kCVPixelFormatType_32BGRA as i64),
    )]);
    unsafe {
        session.beginConfiguration();
        if session.canSetSessionPreset(AVCaptureSessionPreset1280x720) {
            session.setSessionPreset(AVCaptureSessionPreset1280x720);
        }
        output.setVideoSettings(Some(&video_settings));
        output.setAlwaysDiscardsLateVideoFrames(true);
        output.setSampleBufferDelegate_queue(Some(ProtocolObject::from_ref(handler)), Some(queue));
    }
    let can_attach = unsafe { session.canAddInput(input) && session.canAddOutput(output) };
    if !can_attach {
        unsafe { session.commitConfiguration() };
        bail!(CAMERA_BUSY_ERROR);
    }
    unsafe {
        session.addInput(input);
        session.addOutput(output);
        session.commitConfiguration();
    }
    Ok(())
}

impl ActiveCapture for AvFoundationCameraCapture {
    fn stop(self: Box<Self>) -> Result<()> {
        unsafe { self.session.stopRunning() };
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CAMERA_CAPTURE_STOPPED_EVENT})
        );
        Ok(())
    }
}
