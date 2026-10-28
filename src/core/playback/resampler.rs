pub fn convert_channels(samples: &[f32], source_channels: u16, target_channels: u16) -> Vec<f32> {
    let source = source_channels.max(1) as usize;
    let target = target_channels.max(1) as usize;
    if source == target {
        return samples.to_vec();
    }
    samples
        .chunks_exact(source)
        .flat_map(|frame| {
            let mono = frame.iter().sum::<f32>() / source as f32;
            let is_mono_conversion = source == 1 || target == 1;
            (0..target).map(move |channel| match is_mono_conversion {
                true => mono,
                false => frame.get(channel).copied().unwrap_or(mono),
            })
        })
        .collect()
}

pub fn resample_linear(
    samples: &[f32],
    channel_count: u16,
    source_rate: u32,
    target_rate: u32,
) -> Vec<f32> {
    let channels = channel_count.max(1) as usize;
    if source_rate == target_rate || source_rate == 0 || target_rate == 0 {
        return samples.to_vec();
    }
    let source_frames = samples.len() / channels;
    if source_frames == 0 {
        return Vec::new();
    }
    let target_frames = (source_frames as u64 * target_rate as u64 / source_rate as u64) as usize;
    let step = source_rate as f64 / target_rate as f64;
    (0..target_frames)
        .flat_map(|target_index| {
            let source_position = target_index as f64 * step;
            let left_index = (source_position.floor() as usize).min(source_frames - 1);
            let right_index = (left_index + 1).min(source_frames - 1);
            let fraction = (source_position - left_index as f64) as f32;
            (0..channels).map(move |channel| {
                let left = samples[left_index * channels + channel];
                let right = samples[right_index * channels + channel];
                left + (right - left) * fraction
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_to_stereo_duplicates_samples() {
        assert_eq!(
            convert_channels(&[0.1, 0.2], 1, 2),
            vec![0.1, 0.1, 0.2, 0.2]
        );
    }

    #[test]
    fn test_stereo_to_mono_averages() {
        assert_eq!(convert_channels(&[0.2, 0.4], 2, 1), vec![0.3]);
    }

    #[test]
    fn test_resample_doubles_frame_count_when_upsampling() {
        let output = resample_linear(&[0.0, 1.0], 1, 24_000, 48_000);
        assert_eq!(output.len(), 4);
        assert_eq!(output[0], 0.0);
        assert!((output[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_resample_is_identity_for_equal_rates() {
        assert_eq!(
            resample_linear(&[0.5, -0.5], 2, 48_000, 48_000),
            vec![0.5, -0.5]
        );
    }
}
