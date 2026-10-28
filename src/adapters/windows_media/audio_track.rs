use anyhow::{bail, Result};
use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, IMFSinkWriter, MFAudioFormat_AAC, MFAudioFormat_PCM, MFCreateMemoryBuffer,
    MFCreateSample, MFMediaType_Audio, MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
    MF_MT_AUDIO_BITS_PER_SAMPLE, MF_MT_AUDIO_BLOCK_ALIGNMENT, MF_MT_AUDIO_NUM_CHANNELS,
    MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
};

use super::runtime::{create_media_type, HUNDRED_NANOSECONDS_PER_SECOND};
use crate::core::recording::wav_reader::{to_signed_sixteen_bit_bytes, DecodedAudio};

const AAC_SUPPORTED_SAMPLE_RATES: [u32; 2] = [44_100, 48_000];
const AAC_BYTES_PER_SECOND: u32 = 24_000;
const PCM_BITS_PER_SAMPLE: u32 = 16;
const PCM_BYTES_PER_SAMPLE: u32 = 2;
const FRAMES_PER_CHUNK: usize = 1_024;

pub struct AudioTrackWriter {
    stream_index: u32,
    audio: DecodedAudio,
    next_frame: usize,
}

fn build_audio_type(
    subtype: &windows::core::GUID,
    audio: &DecodedAudio,
    is_compressed: bool,
) -> Result<IMFMediaType> {
    let media_type = create_media_type()?;
    let channel_count = audio.channel_count as u32;
    let block_alignment = channel_count * PCM_BYTES_PER_SAMPLE;
    let average_bytes = match is_compressed {
        true => AAC_BYTES_PER_SECOND,
        false => audio.sample_rate * block_alignment,
    };
    unsafe {
        media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        media_type.SetGUID(&MF_MT_SUBTYPE, subtype)?;
        media_type.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, PCM_BITS_PER_SAMPLE)?;
        media_type.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, audio.sample_rate)?;
        media_type.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, channel_count)?;
        media_type.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, average_bytes)?;
        if !is_compressed {
            media_type.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, block_alignment)?;
        }
    }
    Ok(media_type)
}

impl AudioTrackWriter {
    pub fn attach(writer: &IMFSinkWriter, audio: DecodedAudio) -> Result<Self> {
        if !AAC_SUPPORTED_SAMPLE_RATES.contains(&audio.sample_rate)
            || !(1..=2).contains(&audio.channel_count)
        {
            bail!(
                "microphone format {} Hz / {} channels can't be encoded",
                audio.sample_rate,
                audio.channel_count
            );
        }
        let output_type = build_audio_type(&MFAudioFormat_AAC, &audio, true)?;
        let input_type = build_audio_type(&MFAudioFormat_PCM, &audio, false)?;
        let stream_index = unsafe { writer.AddStream(&output_type)? };
        unsafe { writer.SetInputMediaType(stream_index, &input_type, None)? };
        Ok(Self {
            stream_index,
            audio,
            next_frame: 0,
        })
    }

    fn written_time(&self) -> i64 {
        self.next_frame as i64 * HUNDRED_NANOSECONDS_PER_SECOND / self.audio.sample_rate as i64
    }

    pub fn write_until(&mut self, writer: &IMFSinkWriter, timestamp: i64) -> Result<usize> {
        let channel_count = self.audio.channel_count as usize;
        let total_frames = self.audio.interleaved_samples.len() / channel_count.max(1);
        let mut chunks_written = 0usize;
        while self.written_time() < timestamp && self.next_frame < total_frames {
            let end_frame = (self.next_frame + FRAMES_PER_CHUNK).min(total_frames);
            let samples = &self.audio.interleaved_samples
                [self.next_frame * channel_count..end_frame * channel_count];
            self.write_chunk(writer, samples, end_frame - self.next_frame)?;
            self.next_frame = end_frame;
            chunks_written += 1;
        }
        Ok(chunks_written)
    }

    fn write_chunk(
        &self,
        writer: &IMFSinkWriter,
        samples: &[f32],
        frame_count: usize,
    ) -> Result<()> {
        let bytes = to_signed_sixteen_bit_bytes(samples);
        let sample_rate = self.audio.sample_rate as i64;
        unsafe {
            let buffer = MFCreateMemoryBuffer(bytes.len() as u32)?;
            let mut destination: *mut u8 = std::ptr::null_mut();
            buffer.Lock(&mut destination, None, None)?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
            buffer.Unlock()?;
            buffer.SetCurrentLength(bytes.len() as u32)?;
            let sample = MFCreateSample()?;
            sample.AddBuffer(&buffer)?;
            sample.SetSampleTime(self.written_time())?;
            sample.SetSampleDuration(
                frame_count as i64 * HUNDRED_NANOSECONDS_PER_SECOND / sample_rate,
            )?;
            writer.WriteSample(self.stream_index, &sample)?;
        }
        Ok(())
    }
}
