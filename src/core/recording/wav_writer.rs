use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const WAV_HEADER_LENGTH: usize = 44;
const IEEE_FLOAT_FORMAT_TAG: u16 = 3;
const BITS_PER_FLOAT_SAMPLE: u16 = 32;
const BYTES_PER_FLOAT_SAMPLE: u32 = 4;
const FORMAT_CHUNK_LENGTH: u32 = 16;
const RIFF_CHUNK_OVERHEAD: u32 = 36;
const RIFF_MARKER: &[u8; 4] = b"RIFF";
const WAVE_MARKER: &[u8; 4] = b"WAVE";
const FORMAT_MARKER: &[u8; 4] = b"fmt ";
const DATA_MARKER: &[u8; 4] = b"data";
const WAV_WRITER_FLUSH_ERROR: &str = "failed to flush wav writer";

pub fn build_float_wav_header(
    sample_rate: u32,
    channel_count: u16,
    data_length: u32,
) -> [u8; WAV_HEADER_LENGTH] {
    let block_align = channel_count as u32 * BYTES_PER_FLOAT_SAMPLE;
    let byte_rate = sample_rate * block_align;
    let mut header = [0u8; WAV_HEADER_LENGTH];
    header[0..4].copy_from_slice(RIFF_MARKER);
    header[4..8].copy_from_slice(&(RIFF_CHUNK_OVERHEAD + data_length).to_le_bytes());
    header[8..12].copy_from_slice(WAVE_MARKER);
    header[12..16].copy_from_slice(FORMAT_MARKER);
    header[16..20].copy_from_slice(&FORMAT_CHUNK_LENGTH.to_le_bytes());
    header[20..22].copy_from_slice(&IEEE_FLOAT_FORMAT_TAG.to_le_bytes());
    header[22..24].copy_from_slice(&channel_count.to_le_bytes());
    header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&(block_align as u16).to_le_bytes());
    header[34..36].copy_from_slice(&BITS_PER_FLOAT_SAMPLE.to_le_bytes());
    header[36..40].copy_from_slice(DATA_MARKER);
    header[40..44].copy_from_slice(&data_length.to_le_bytes());
    header
}

pub struct WavFileWriter {
    file_path: PathBuf,
    writer: BufWriter<File>,
    sample_rate: u32,
    channel_count: u16,
    data_length: u32,
}

impl WavFileWriter {
    pub fn create(file_path: &Path, sample_rate: u32, channel_count: u16) -> Result<Self> {
        let file =
            File::create(file_path).with_context(|| format!("failed to create {:?}", file_path))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&build_float_wav_header(sample_rate, channel_count, 0))?;
        Ok(Self {
            file_path: file_path.to_path_buf(),
            writer,
            sample_rate,
            channel_count,
            data_length: 0,
        })
    }

    pub fn format_matches(&self, sample_rate: u32, channel_count: u16) -> bool {
        self.sample_rate == sample_rate && self.channel_count == channel_count
    }

    pub fn append_samples(&mut self, interleaved_samples: &[f32]) -> Result<u32> {
        for sample in interleaved_samples {
            self.writer.write_all(&sample.to_le_bytes())?;
        }
        let appended_length = interleaved_samples.len() as u32 * BYTES_PER_FLOAT_SAMPLE;
        self.data_length = self.data_length.saturating_add(appended_length);
        Ok(self.data_length)
    }

    pub fn has_audio(&self) -> bool {
        self.data_length > 0
    }

    pub fn finalize(mut self) -> Result<PathBuf> {
        let header = build_float_wav_header(self.sample_rate, self.channel_count, self.data_length);
        self.writer.flush()?;
        let mut file = self.writer.into_inner().context(WAV_WRITER_FLUSH_ERROR)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&header)?;
        file.sync_all()?;
        Ok(self.file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_describes_float_stereo_audio() {
        let header = build_float_wav_header(48_000, 2, 800);
        assert_eq!(&header[0..4], b"RIFF");
        assert_eq!(
            u32::from_le_bytes([header[4], header[5], header[6], header[7]]),
            836
        );
        assert_eq!(u16::from_le_bytes([header[20], header[21]]), 3);
        assert_eq!(u16::from_le_bytes([header[22], header[23]]), 2);
        assert_eq!(
            u32::from_le_bytes([header[28], header[29], header[30], header[31]]),
            384_000
        );
        assert_eq!(
            u32::from_le_bytes([header[40], header[41], header[42], header[43]]),
            800
        );
    }

    #[test]
    fn test_writer_finalizes_with_correct_data_length() {
        let file_path = std::env::temp_dir().join(format!("cts_wav_{}.wav", uuid::Uuid::new_v4()));
        let mut writer = WavFileWriter::create(&file_path, 44_100, 1).expect("create wav");
        assert!(!writer.has_audio());
        let _ = writer.append_samples(&[0.0, 0.5, -0.5]);
        assert!(writer.format_matches(44_100, 1));
        assert!(!writer.format_matches(48_000, 1));
        let finalized_path = writer.finalize().expect("finalize wav");
        let bytes = std::fs::read(&finalized_path).unwrap_or_default();
        assert_eq!(bytes.len(), 44 + 12);
        assert_eq!(
            u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]),
            12
        );
        let _ = std::fs::remove_file(finalized_path);
    }
}
