use super::CalibrationMetrics;
use crate::error::LogiscoreError;

const MAX_ANALYSIS_SAMPLES: usize = 16_000_000;

pub fn analyze_calibration(
    samples: &[f32],
    sample_rate: u32,
) -> Result<CalibrationMetrics, LogiscoreError> {
    if !(8_000..=384_000).contains(&sample_rate)
        || samples.is_empty()
        || samples.len() > MAX_ANALYSIS_SAMPLES
        || samples.iter().any(|sample| !sample.is_finite())
    {
        return Err(LogiscoreError::InvalidProfile(
            "calibration audio is empty, invalid, or outside supported limits".into(),
        ));
    }
    let onset = samples
        .iter()
        .position(|sample| sample.abs() >= 0.02)
        .unwrap_or(samples.len());
    let maximum_noise_window = usize::try_from(sample_rate / 20).unwrap_or(usize::MAX);
    let noise_end = onset.min(maximum_noise_window).max(1).min(samples.len());
    let noise_rms = rms(&samples[..noise_end]);
    let signal_rms = rms(&samples[onset.min(samples.len())..]);
    let noise_floor_db = amplitude_db(noise_rms);
    let snr_db = (amplitude_db(signal_rms) - noise_floor_db).clamp(-20.0, 120.0);
    let clipping_ratio =
        samples.iter().filter(|sample| sample.abs() >= 0.98).count() as f32 / samples.len() as f32;
    let trailing_quiet = samples
        .iter()
        .rev()
        .take_while(|sample| sample.abs() < 0.02)
        .count();
    let reverberation_ms = ((u64::try_from(trailing_quiet).unwrap_or(u64::MAX) * 1_000)
        / u64::from(sample_rate))
    .min(5_000) as u16;
    CalibrationMetrics::new(noise_floor_db, snr_db, clipping_ratio, reverberation_ms)
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum = samples.iter().map(|sample| sample * sample).sum::<f32>();
    (sum / samples.len() as f32).sqrt()
}

fn amplitude_db(amplitude: f32) -> f32 {
    if amplitude <= f32::EPSILON {
        -120.0
    } else {
        (20.0 * amplitude.log10()).clamp(-120.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_measures_noise_signal_and_clipping() {
        let mut samples = vec![0.001; 400];
        samples.extend(vec![0.5; 800]);
        samples.push(1.0);
        let metrics = analyze_calibration(&samples, 8_000).unwrap();
        assert!(metrics.noise_floor_db < -50.0);
        assert!(metrics.snr_db > 40.0);
        assert!(metrics.clipping_ratio > 0.0);
    }

    #[test]
    fn calibration_rejects_invalid_audio() {
        assert!(analyze_calibration(&[], 8_000).is_err());
        assert!(analyze_calibration(&[f32::NAN], 8_000).is_err());
        assert!(analyze_calibration(&[0.0], 7_999).is_err());
    }
}
