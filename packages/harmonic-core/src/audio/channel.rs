use super::MAX_PCM_SAMPLES;
use crate::error::LogiscoreError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelModel {
    pub gain: f32,
    pub noise_amplitude: f32,
    pub clip_level: f32,
    pub leading_silence_samples: usize,
    pub echo_delay_samples: usize,
    pub echo_decay: f32,
}

impl Default for ChannelModel {
    fn default() -> Self {
        Self {
            gain: 1.0,
            noise_amplitude: 0.0,
            clip_level: 1.0,
            leading_silence_samples: 0,
            echo_delay_samples: 0,
            echo_decay: 0.0,
        }
    }
}

impl ChannelModel {
    pub fn apply(self, samples: &[f32]) -> Result<Vec<f32>, LogiscoreError> {
        self.validate(samples)?;
        let echo_tail = usize::from(self.echo_decay > 0.0) * self.echo_delay_samples;
        let output_length = self
            .leading_silence_samples
            .checked_add(samples.len())
            .and_then(|length| length.checked_add(echo_tail))
            .filter(|length| *length <= MAX_PCM_SAMPLES)
            .ok_or_else(|| invalid_audio("channel output exceeds the decoder limit"))?;
        let mut output = vec![0.0; output_length];
        for (index, sample) in samples.iter().copied().enumerate() {
            let output_index = self.leading_silence_samples + index;
            output[output_index] += sample * self.gain;
            if self.echo_decay > 0.0 {
                output[output_index + self.echo_delay_samples] +=
                    sample * self.gain * self.echo_decay;
            }
        }
        let mut noise_state = 0x4c53_4352u32;
        for sample in &mut output {
            noise_state = noise_state
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223);
            let normalized = ((noise_state >> 8) as f32 / 0x00ff_ffff as f32) * 2.0 - 1.0;
            *sample = (*sample + normalized * self.noise_amplitude)
                .clamp(-self.clip_level, self.clip_level);
        }
        Ok(output)
    }

    fn validate(self, samples: &[f32]) -> Result<(), LogiscoreError> {
        if samples.iter().any(|sample| !sample.is_finite())
            || !self.gain.is_finite()
            || !self.noise_amplitude.is_finite()
            || !self.clip_level.is_finite()
            || !self.echo_decay.is_finite()
        {
            return Err(invalid_audio("channel parameters must be finite"));
        }
        if self.gain <= 0.0
            || self.noise_amplitude < 0.0
            || self.clip_level <= 0.0
            || self.clip_level > 1.0
            || self.echo_decay < 0.0
            || self.echo_decay > 1.0
            || (self.echo_decay > 0.0 && self.echo_delay_samples == 0)
        {
            return Err(invalid_audio(
                "channel parameters are outside supported ranges",
            ));
        }
        Ok(())
    }
}

fn invalid_audio(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidAudio(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_channel_conditions_preserve_reliable_payload() {
        let samples = crate::v2::encode_text_pcm_reliable("room channel").unwrap();
        let degraded = ChannelModel {
            gain: 0.72,
            noise_amplitude: 0.002,
            clip_level: 0.32,
            leading_silence_samples: 173,
            echo_delay_samples: 32,
            echo_decay: 0.08,
        }
        .apply(&samples)
        .unwrap();
        assert_eq!(
            crate::v2::decode_text_pcm_reliable(&degraded).unwrap(),
            "room channel"
        );
    }

    #[test]
    fn rejects_unsafe_parameters() {
        assert!(ChannelModel {
            gain: 0.0,
            ..ChannelModel::default()
        }
        .apply(&[0.0])
        .is_err());
        assert!(ChannelModel {
            echo_decay: 0.5,
            ..ChannelModel::default()
        }
        .apply(&[0.0])
        .is_err());
    }
}
