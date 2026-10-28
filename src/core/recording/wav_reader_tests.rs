use super::wav_reader::{
    mix_tracks, parse_float_wav, read_float_wav, to_signed_sixteen_bit_bytes, DecodedAudio,
};
use crate::core::recording::wav_writer::WavFileWriter;

#[test]
fn test_round_trip_with_wav_writer() {
    let path = std::env::temp_dir().join(format!("cts_wav_read_{}.wav", uuid::Uuid::new_v4()));
    let mut writer = WavFileWriter::create(&path, 48_000, 2).expect("create");
    let _ = writer.append_samples(&[0.25, -0.25, 0.5, -0.5]);
    let path = writer.finalize().expect("finalize");
    let decoded = read_float_wav(&path).expect("read");
    assert_eq!(
        decoded,
        DecodedAudio {
            sample_rate: 48_000,
            channel_count: 2,
            interleaved_samples: vec![0.25, -0.25, 0.5, -0.5]
        }
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_rejects_non_wav_data() {
    assert!(parse_float_wav(b"not audio at all").is_err());
}

#[test]
fn test_mix_tracks_sums_and_pads() {
    let first = DecodedAudio {
        sample_rate: 48_000,
        channel_count: 1,
        interleaved_samples: vec![0.5, 0.5, 0.5],
    };
    let second = DecodedAudio {
        sample_rate: 48_000,
        channel_count: 1,
        interleaved_samples: vec![0.25, 0.75],
    };
    let mixed = mix_tracks(vec![first, second]).expect("mix");
    assert_eq!(mixed.interleaved_samples, vec![0.75, 1.0, 0.5]);
}

#[test]
fn test_mix_tracks_rejects_different_formats() {
    let first = DecodedAudio {
        sample_rate: 48_000,
        channel_count: 1,
        interleaved_samples: vec![0.0],
    };
    let second = DecodedAudio {
        sample_rate: 44_100,
        channel_count: 1,
        interleaved_samples: vec![0.0],
    };
    assert!(mix_tracks(vec![first, second]).is_err());
}

#[test]
fn test_sixteen_bit_conversion_clamps() {
    assert_eq!(
        to_signed_sixteen_bit_bytes(&[2.0, -2.0, 0.0]),
        [
            32767i16.to_le_bytes(),
            (-32767i16).to_le_bytes(),
            0i16.to_le_bytes()
        ]
        .concat()
    );
}
