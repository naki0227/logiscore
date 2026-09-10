use crate::error::LogiscoreError;

pub(super) fn linear(
    samples: &[f32],
    source_rate: u32,
    target_rate: u32,
    max_output_samples: usize,
) -> Result<Vec<f32>, LogiscoreError> {
    if source_rate == 0 || target_rate == 0 {
        return Err(invalid_audio("sample rate must be positive"));
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(invalid_audio("PCM contains non-finite samples"));
    }
    if samples.is_empty() {
        return Ok(Vec::new());
    }
    if source_rate == target_rate {
        if samples.len() > max_output_samples {
            return Err(invalid_audio("resampled PCM exceeds the decoder limit"));
        }
        return Ok(samples.to_vec());
    }
    let output_length = u64::try_from(samples.len())
        .ok()
        .and_then(|length| length.checked_mul(u64::from(target_rate)))
        .and_then(|value| value.checked_add(u64::from(source_rate) - 1))
        .map(|value| value / u64::from(source_rate))
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid_audio("resampled length overflow"))?;
    if output_length > max_output_samples {
        return Err(invalid_audio("resampled PCM exceeds the decoder limit"));
    }

    let ratio = source_rate as f64 / target_rate as f64;
    Ok((0..output_length)
        .map(|output_index| {
            let source_position = output_index as f64 * ratio;
            let left_index = (source_position.floor() as usize).min(samples.len() - 1);
            let right_index = (left_index + 1).min(samples.len() - 1);
            let fraction = (source_position - left_index as f64) as f32;
            samples[left_index] * (1.0 - fraction) + samples[right_index] * fraction
        })
        .collect())
}

fn invalid_audio(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidAudio(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_duration_when_resampling() {
        let input = vec![0.5; 8_000];
        let upsampled = linear(&input, 8_000, 48_000, 100_000).unwrap();
        assert_eq!(upsampled.len(), 48_000);
        assert!(upsampled.iter().all(|sample| *sample == 0.5));
    }

    #[test]
    fn rejects_invalid_rate_and_output_limit() {
        assert!(linear(&[0.0], 0, 8_000, 10).is_err());
        assert!(linear(&[0.0; 10], 8_000, 48_000, 10).is_err());
    }
}
