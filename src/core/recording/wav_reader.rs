#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

use std::path::Path;

use anyhow::{bail, Context, Result};

const RIFF_MARKER: &[u8; 4] = b"RIFF";
const WAVE_MARKER: &[u8; 4] = b"WAVE";
const FORMAT_MARKER: &[u8; 4] = b"fmt ";
const DATA_MARKER: &[u8; 4] = b"data";
const IEEE_FLOAT_FORMAT_TAG: u16 = 3;
const CHUNK_HEADER_LENGTH: usize = 8;
const RIFF_HEADER_LENGTH: usize = 12;
const MINIMUM_FORMAT_BODY_LENGTH: usize = 8;
const BYTES_PER_FLOAT_SAMPLE: usize = 4;
const NOT_A_WAV_FILE_ERROR: &str = "not a WAV file";
const DATA_BEFORE_FORMAT_ERROR: &str = "WAV data appears before its format";
const UNSUPPORTED_SAMPLE_FORMAT_ERROR: &str = "only 32-bit float WAV audio is supported";
const MISSING_AUDIO_DATA_ERROR: &str = "WAV file has no audio data";
const NO_TRACKS_TO_MIX_ERROR: &str = "no audio tracks to mix";
const MISMATCHED_TRACK_FORMATS_ERROR: &str = "audio tracks have different formats";

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedAudio {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub interleaved_samples: Vec<f32>,
}

struct WavFormat {
    format_tag: u16,
    channel_count: u16,
    sample_rate: u32,
}

struct ChunkBounds<'bytes> {
    marker: &'bytes [u8],
    body_start: usize,
    body_end: usize,
    next_offset: usize,
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

pub fn parse_float_wav(bytes: &[u8]) -> Result<DecodedAudio> {
    if !has_riff_wave_header(bytes) {
        bail!(NOT_A_WAV_FILE_ERROR);
    }
    let mut offset = RIFF_HEADER_LENGTH;
    let mut format: Option<WavFormat> = None;
    while offset + CHUNK_HEADER_LENGTH <= bytes.len() {
        let chunk = read_chunk_bounds(bytes, offset);
        if chunk.marker == DATA_MARKER {
            return decode_data_chunk(bytes, &chunk, format);
        }
        format = read_format_chunk(bytes, &chunk).or(format);
        offset = chunk.next_offset;
    }
    bail!(MISSING_AUDIO_DATA_ERROR)
}

fn has_riff_wave_header(bytes: &[u8]) -> bool {
    bytes.len() >= RIFF_HEADER_LENGTH && &bytes[0..4] == RIFF_MARKER && &bytes[8..12] == WAVE_MARKER
}

fn read_chunk_bounds(bytes: &[u8], offset: usize) -> ChunkBounds<'_> {
    let chunk_length = read_u32(bytes, offset + 4) as usize;
    let body_start = offset + CHUNK_HEADER_LENGTH;
    ChunkBounds {
        marker: &bytes[offset..offset + 4],
        body_start,
        body_end: (body_start + chunk_length).min(bytes.len()),
        next_offset: body_start + chunk_length + (chunk_length % 2),
    }
}

fn read_format_chunk(bytes: &[u8], chunk: &ChunkBounds) -> Option<WavFormat> {
    let is_complete_format_chunk = chunk.marker == FORMAT_MARKER
        && chunk.body_end - chunk.body_start >= MINIMUM_FORMAT_BODY_LENGTH;
    is_complete_format_chunk.then(|| WavFormat {
        format_tag: read_u16(bytes, chunk.body_start),
        channel_count: read_u16(bytes, chunk.body_start + 2),
        sample_rate: read_u32(bytes, chunk.body_start + 4),
    })
}

fn decode_data_chunk(
    bytes: &[u8],
    chunk: &ChunkBounds,
    format: Option<WavFormat>,
) -> Result<DecodedAudio> {
    let format = format.context(DATA_BEFORE_FORMAT_ERROR)?;
    if format.format_tag != IEEE_FLOAT_FORMAT_TAG {
        bail!(UNSUPPORTED_SAMPLE_FORMAT_ERROR);
    }
    let interleaved_samples = bytes[chunk.body_start..chunk.body_end]
        .chunks_exact(BYTES_PER_FLOAT_SAMPLE)
        .map(|sample| f32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]))
        .collect();
    Ok(DecodedAudio {
        sample_rate: format.sample_rate,
        channel_count: format.channel_count,
        interleaved_samples,
    })
}

pub fn read_float_wav(path: &Path) -> Result<DecodedAudio> {
    let bytes = std::fs::read(path).with_context(|| format!("could not read {:?}", path))?;
    parse_float_wav(&bytes)
}

pub fn mix_tracks(tracks: Vec<DecodedAudio>) -> Result<DecodedAudio> {
    let mut remaining = tracks.into_iter();
    let first = remaining.next().context(NO_TRACKS_TO_MIX_ERROR)?;
    remaining.try_fold(first, |mixed, track| {
        if track.sample_rate != mixed.sample_rate || track.channel_count != mixed.channel_count {
            bail!(MISMATCHED_TRACK_FORMATS_ERROR);
        }
        let length = mixed
            .interleaved_samples
            .len()
            .max(track.interleaved_samples.len());
        let sample_at = |samples: &[f32], index: usize| samples.get(index).copied().unwrap_or(0.0);
        let interleaved_samples = (0..length)
            .map(|index| {
                (sample_at(&mixed.interleaved_samples, index)
                    + sample_at(&track.interleaved_samples, index))
                .clamp(-1.0, 1.0)
            })
            .collect();
        Ok(DecodedAudio {
            interleaved_samples,
            ..mixed
        })
    })
}

pub fn to_signed_sixteen_bit_bytes(samples: &[f32]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| ((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes())
        .collect()
}
