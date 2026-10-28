use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::core::interfaces::ports::{
    ActiveCapture, AudioInputSource, MediaSink, ScreenFrameSource,
};
use crate::core::models::{
    AudioChunk, AudioTrackKind, OutputDimensions, PixelLayout, ScreenCaptureRequest, VideoFrame,
};

struct ThreadCapture {
    running: Arc<AtomicBool>,
}

impl ActiveCapture for ThreadCapture {
    fn stop(self: Box<Self>) -> anyhow::Result<()> {
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }
}

fn spawn_producer(
    mut produce: impl FnMut(u64) -> bool + Send + 'static,
    interval: Duration,
) -> Box<dyn ActiveCapture> {
    let running = Arc::new(AtomicBool::new(true));
    let producer_running = Arc::clone(&running);
    std::thread::spawn(move || {
        let mut tick = 0u64;
        while producer_running.load(Ordering::SeqCst) {
            let _ = produce(tick);
            tick += 1;
            std::thread::sleep(interval);
        }
    });
    Box::new(ThreadCapture { running })
}

pub(super) struct SyntheticScreen;
impl ScreenFrameSource for SyntheticScreen {
    fn start_capture(
        &self,
        request: ScreenCaptureRequest,
        sink: Arc<dyn MediaSink>,
    ) -> anyhow::Result<Box<dyn ActiveCapture>> {
        let OutputDimensions { width, height } = request.output;
        let emits_system_audio = request.captures_system_audio;
        Ok(spawn_producer(
            move |tick| {
                if emits_system_audio {
                    let samples = vec![0.1f32; 1_600 * 2];
                    sink.accept_audio_chunk(
                        AudioTrackKind::SystemAudio,
                        AudioChunk {
                            sample_rate: 48_000,
                            channel_count: 2,
                            interleaved_samples: samples,
                        },
                    );
                }
                let luminance = (tick * 7 % 255) as u8;
                let mut data =
                    vec![luminance; PixelLayout::Nv12.expected_byte_length(width, height)];
                let chroma_start = (width * height) as usize;
                data[chroma_start..].fill(128);
                sink.accept_video_frame(VideoFrame {
                    width,
                    height,
                    pixel_layout: PixelLayout::Nv12,
                    data: Arc::new(data),
                })
            },
            Duration::from_millis(33),
        ))
    }
}

pub(super) struct SyntheticMicrophone;
impl AudioInputSource for SyntheticMicrophone {
    fn start_capture(&self, sink: Arc<dyn MediaSink>) -> anyhow::Result<Box<dyn ActiveCapture>> {
        let started_at = std::time::Instant::now();
        let mut samples_emitted: u64 = 0;
        Ok(spawn_producer(
            move |_tick| {
                let samples_due = (started_at.elapsed().as_secs_f64() * 48_000.0) as u64;
                let samples: Vec<f32> = (samples_emitted..samples_due)
                    .map(|index| {
                        (index as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 0.2
                    })
                    .collect();
                samples_emitted = samples_due;
                sink.accept_audio_chunk(
                    AudioTrackKind::Microphone,
                    AudioChunk {
                        sample_rate: 48_000,
                        channel_count: 1,
                        interleaved_samples: samples,
                    },
                )
            },
            Duration::from_millis(20),
        ))
    }
}
