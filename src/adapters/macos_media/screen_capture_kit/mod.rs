mod configuration;
mod content;
mod output_handler;
mod sample_conversion;

use std::sync::Arc;

use anyhow::{anyhow, Result};
use dispatch2::{DispatchQueue, DispatchQueueAttr, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::AllocAnyThread;
use objc2_screen_capture_kit::{SCStream, SCStreamOutputType};

use super::foundation_helpers::{describe_error, wait_for_error_completion};
use crate::core::interfaces::ports::{ActiveCapture, MediaSink, ScreenFrameSource};
use crate::core::models::ScreenCaptureRequest;
use output_handler::StreamOutputHandler;

const LOG_TAG: &str = "[SCREEN_CAPTURE_KIT]";
const SAMPLE_QUEUE_LABEL: &str = "circle-to-search.screen-capture";
const SCREEN_CAPTURE_STARTED_EVENT: &str = "screen_capture_started";
const SCREEN_CAPTURE_STOPPED_EVENT: &str = "screen_capture_stopped";

pub struct ScreenCaptureKitSource;

struct ScreenCaptureKitCapture {
    stream: Retained<SCStream>,
    _output_handler: Retained<StreamOutputHandler>,
    _sample_queue: DispatchRetained<DispatchQueue>,
}

unsafe impl Send for ScreenCaptureKitCapture {}

impl ScreenFrameSource for ScreenCaptureKitSource {
    fn start_capture(
        &self,
        request: ScreenCaptureRequest,
        sink: Arc<dyn MediaSink>,
    ) -> Result<Box<dyn ActiveCapture>> {
        let stream = build_filtered_stream(&request)?;
        let output_handler = StreamOutputHandler::build(sink);
        let sample_queue = DispatchQueue::new(SAMPLE_QUEUE_LABEL, DispatchQueueAttr::SERIAL);
        attach_requested_outputs(&stream, &output_handler, &sample_queue, &request)?;
        wait_for_error_completion(|completion| unsafe {
            stream.startCaptureWithCompletionHandler(Some(completion))
        })?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SCREEN_CAPTURE_STARTED_EVENT, "width": request.output.width, "height": request.output.height, "fps": request.frames_per_second})
        );
        Ok(Box::new(ScreenCaptureKitCapture {
            stream,
            _output_handler: output_handler,
            _sample_queue: sample_queue,
        }))
    }
}

fn attach_output(
    stream: &SCStream,
    handler: &StreamOutputHandler,
    queue: &DispatchQueue,
    output_type: SCStreamOutputType,
) -> Result<()> {
    unsafe {
        stream.addStreamOutput_type_sampleHandlerQueue_error(
            ProtocolObject::from_ref(handler),
            output_type,
            Some(queue),
        )
    }
    .map_err(|error| {
        anyhow!(
            "could not attach capture output: {}",
            describe_error(&error)
        )
    })
}

impl ActiveCapture for ScreenCaptureKitCapture {
    fn stop(self: Box<Self>) -> Result<()> {
        let stream = self.stream.clone();
        let stop_result = wait_for_error_completion(|completion| unsafe {
            stream.stopCaptureWithCompletionHandler(Some(completion))
        });
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SCREEN_CAPTURE_STOPPED_EVENT, "ok": stop_result.is_ok()})
        );
        stop_result
    }
}

fn build_filtered_stream(request: &ScreenCaptureRequest) -> Result<Retained<SCStream>> {
    let shareable_content = content::fetch_shareable_content()?;
    let display = content::find_display(
        &shareable_content,
        request.display_origin_x,
        request.display_origin_y,
    )?;
    let filter = content::build_content_filter(
        &shareable_content,
        &display,
        &request.visible_own_window_titles,
    );
    let stream_configuration = configuration::build_stream_configuration(request, &display);
    Ok(unsafe {
        SCStream::initWithFilter_configuration_delegate(
            SCStream::alloc(),
            &filter,
            &stream_configuration,
            None,
        )
    })
}

fn attach_requested_outputs(
    stream: &SCStream,
    output_handler: &StreamOutputHandler,
    sample_queue: &DispatchQueue,
    request: &ScreenCaptureRequest,
) -> Result<bool> {
    attach_output(
        stream,
        output_handler,
        sample_queue,
        SCStreamOutputType::Screen,
    )?;
    if !request.captures_system_audio {
        return Ok(false);
    }
    attach_output(
        stream,
        output_handler,
        sample_queue,
        SCStreamOutputType::Audio,
    )?;
    Ok(true)
}
