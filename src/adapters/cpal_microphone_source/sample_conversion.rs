const UNSIGNED_SIXTEEN_BIT_MIDPOINT: f32 = 32_768.0;

pub(super) fn convert_signed_samples(data: &[i16]) -> Vec<f32> {
    data.iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect()
}

pub(super) fn convert_unsigned_samples(data: &[u16]) -> Vec<f32> {
    data.iter()
        .map(|sample| {
            (*sample as f32 - UNSIGNED_SIXTEEN_BIT_MIDPOINT) / UNSIGNED_SIXTEEN_BIT_MIDPOINT
        })
        .collect()
}
