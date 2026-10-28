use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use super::audio_ring::AudioRing;
use crate::core::interfaces::ports::{AudioTrackDecoder, TimedVideoFrame, VideoTrackDecoder};

const VIDEO_QUEUE_CAPACITY: usize = 6;
const AUDIO_WAIT_INTERVAL: Duration = Duration::from_millis(10);
const LOG_TAG: &str = "[PLAYBACK]";
const VIDEO_DECODE_FAILED_EVENT: &str = "video_decode_failed";
const AUDIO_DECODE_FAILED_EVENT: &str = "audio_decode_failed";

pub struct VideoPump {
    pub frames: Receiver<TimedVideoFrame>,
    stop_requested: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl VideoPump {
    pub fn start(mut decoder: Box<dyn VideoTrackDecoder>) -> Self {
        let (sender, frames) = mpsc::sync_channel::<TimedVideoFrame>(VIDEO_QUEUE_CAPACITY);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop_requested);
        let worker = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::SeqCst) {
                match decoder.next_frame() {
                    Ok(Some(frame)) => {
                        if sender.send(frame).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(decode_error) => {
                        log::error!(
                            "{} {}",
                            LOG_TAG,
                            serde_json::json!({"event": VIDEO_DECODE_FAILED_EVENT, "error": decode_error.to_string()})
                        );
                        break;
                    }
                }
            }
        });
        Self {
            frames,
            stop_requested,
            worker: Some(worker),
        }
    }
}

impl Drop for VideoPump {
    fn drop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        while self.frames.try_recv().is_ok() {}
        if let Some(worker) = self.worker.take() {
            drop(std::mem::replace(&mut self.frames, mpsc::sync_channel(0).1));
            let _ = worker.join();
        }
    }
}

pub struct AudioPump {
    stop_requested: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl AudioPump {
    pub fn start(mut decoder: Box<dyn AudioTrackDecoder>, ring: Arc<AudioRing>) -> Self {
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop_requested);
        let worker = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::SeqCst) {
                if !ring.has_room() {
                    std::thread::sleep(AUDIO_WAIT_INTERVAL);
                    continue;
                }
                match decoder.next_chunk() {
                    Ok(Some(chunk)) => {
                        ring.push_chunk(&chunk);
                    }
                    Ok(None) => break,
                    Err(decode_error) => {
                        log::error!(
                            "{} {}",
                            LOG_TAG,
                            serde_json::json!({"event": AUDIO_DECODE_FAILED_EVENT, "error": decode_error.to_string()})
                        );
                        break;
                    }
                }
            }
        });
        Self {
            stop_requested,
            worker: Some(worker),
        }
    }
}

impl Drop for AudioPump {
    fn drop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
