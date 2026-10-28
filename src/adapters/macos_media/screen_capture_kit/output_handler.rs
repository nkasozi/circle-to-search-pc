use std::sync::Arc;

use objc2::rc::Retained;
use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass};
use objc2_core_media::CMSampleBuffer;
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_screen_capture_kit::{SCStream, SCStreamOutput, SCStreamOutputType};

use super::sample_conversion::{convert_audio_sample, convert_screen_sample};
use crate::core::interfaces::ports::MediaSink;
use crate::core::models::AudioTrackKind;

pub struct StreamOutputState {
    sink: Arc<dyn MediaSink>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "CircleToSearchStreamOutput"]
    #[ivars = StreamOutputState]
    pub struct StreamOutputHandler;

    unsafe impl NSObjectProtocol for StreamOutputHandler {}

    unsafe impl SCStreamOutput for StreamOutputHandler {
        #[unsafe(method(stream:didOutputSampleBuffer:ofType:))]
        unsafe fn stream_did_output_sample_buffer(
            &self,
            _stream: &SCStream,
            sample_buffer: &CMSampleBuffer,
            output_type: SCStreamOutputType,
        ) {
            let _ = self.forward_sample(sample_buffer, output_type);
        }
    }
);

impl StreamOutputHandler {
    pub fn build(sink: Arc<dyn MediaSink>) -> Retained<Self> {
        let allocated = Self::alloc().set_ivars(StreamOutputState { sink });
        unsafe { msg_send![super(allocated), init] }
    }

    fn forward_sample(
        &self,
        sample_buffer: &CMSampleBuffer,
        output_type: SCStreamOutputType,
    ) -> bool {
        let sink = &self.ivars().sink;
        match output_type {
            SCStreamOutputType::Screen => convert_screen_sample(sample_buffer)
                .map(|frame| sink.accept_video_frame(frame))
                .unwrap_or(false),
            SCStreamOutputType::Audio => convert_audio_sample(sample_buffer)
                .map(|chunk| sink.accept_audio_chunk(AudioTrackKind::SystemAudio, chunk))
                .unwrap_or(false),
            _ => false,
        }
    }
}
