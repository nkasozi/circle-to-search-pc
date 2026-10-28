use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use super::resampler::{convert_channels, resample_linear};
use crate::core::interfaces::ports::PlaybackAudioSource;
use crate::core::models::AudioChunk;

const MAXIMUM_BUFFERED_SECONDS: f32 = 1.0;

#[derive(Default)]
pub struct AudioRing {
    samples: Mutex<VecDeque<f32>>,
    device_format: Mutex<Option<(u32, u16)>>,
    is_playing: AtomicBool,
}

impl AudioRing {
    pub fn set_playing(&self, is_playing: bool) -> bool {
        self.is_playing.swap(is_playing, Ordering::SeqCst)
    }

    pub fn clear(&self) -> usize {
        self.samples
            .lock()
            .map(|mut samples| std::mem::take(&mut *samples).len())
            .unwrap_or(0)
    }

    pub fn device_format(&self) -> Option<(u32, u16)> {
        self.device_format.lock().ok().and_then(|format| *format)
    }

    pub fn has_room(&self) -> bool {
        let Some((sample_rate, channel_count)) = self.device_format() else {
            return false;
        };
        let capacity =
            (sample_rate as f32 * channel_count as f32 * MAXIMUM_BUFFERED_SECONDS) as usize;
        self.samples
            .lock()
            .map(|samples| samples.len() < capacity)
            .unwrap_or(false)
    }

    pub fn push_chunk(&self, chunk: &AudioChunk) -> usize {
        let Some((sample_rate, channel_count)) = self.device_format() else {
            return 0;
        };
        let channel_matched = convert_channels(
            &chunk.interleaved_samples,
            chunk.channel_count,
            channel_count,
        );
        let resampled = resample_linear(
            &channel_matched,
            channel_count,
            chunk.sample_rate,
            sample_rate,
        );
        let pushed = resampled.len();
        if let Ok(mut samples) = self.samples.lock() {
            samples.extend(resampled);
        }
        pushed
    }
}

impl PlaybackAudioSource for AudioRing {
    fn fill(&self, output: &mut [f32], sample_rate: u32, channel_count: u16) -> usize {
        if let Ok(mut format) = self.device_format.lock() {
            format.get_or_insert((sample_rate, channel_count));
        }
        output.fill(0.0);
        if !self.is_playing.load(Ordering::SeqCst) {
            return 0;
        }
        let Ok(mut samples) = self.samples.lock() else {
            return 0;
        };
        let available = samples.len().min(output.len());
        for (slot, sample) in output.iter_mut().zip(samples.drain(..available)) {
            *slot = sample;
        }
        available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(samples: Vec<f32>) -> AudioChunk {
        AudioChunk {
            sample_rate: 48_000,
            channel_count: 1,
            interleaved_samples: samples,
        }
    }

    #[test]
    fn test_nothing_is_buffered_before_the_device_format_is_known() {
        let ring = AudioRing::default();
        assert_eq!(ring.push_chunk(&chunk(vec![0.5])), 0);
        assert!(!ring.has_room());
    }

    #[test]
    fn test_fill_outputs_silence_while_paused_and_samples_while_playing() {
        let ring = AudioRing::default();
        let mut output = vec![9.0; 4];
        ring.fill(&mut output, 48_000, 2);
        assert_eq!(output, vec![0.0; 4]);
        assert_eq!(ring.push_chunk(&chunk(vec![0.25, 0.5])), 4);
        ring.fill(&mut output, 48_000, 2);
        assert_eq!(output, vec![0.0; 4]);
        ring.set_playing(true);
        assert_eq!(ring.fill(&mut output, 48_000, 2), 4);
        assert_eq!(output, vec![0.25, 0.25, 0.5, 0.5]);
    }

    #[test]
    fn test_clear_drops_buffered_audio() {
        let ring = AudioRing::default();
        let mut output = vec![0.0; 2];
        ring.fill(&mut output, 48_000, 1);
        ring.push_chunk(&chunk(vec![0.1, 0.2, 0.3]));
        assert_eq!(ring.clear(), 3);
    }
}
