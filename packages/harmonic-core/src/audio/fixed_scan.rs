use super::fixed::{validate_samples, FixedPcmCodec};
use super::{resample, PcmProfile, MAX_PCM_SAMPLES};
use crate::error::LogiscoreError;

impl FixedPcmCodec {
    pub(crate) fn decode_all_at_sample_rate(
        self,
        samples: &[f32],
        source_sample_rate: u32,
    ) -> Result<Vec<Vec<u8>>, LogiscoreError> {
        let normalized = resample::linear(
            samples,
            source_sample_rate,
            self.profile.sample_rate(),
            MAX_PCM_SAMPLES,
        )?;
        validate_samples(&normalized)?;
        let mut packets = Vec::new();
        let mut consumed_until = 0;
        for sync_start in checkpoint_onsets(&normalized, self.profile) {
            if sync_start < consumed_until {
                continue;
            }
            if let Ok((packet, frame_end)) = self.decode_from_sync(&normalized, sync_start) {
                packets.push(packet);
                consumed_until = frame_end;
            }
        }
        Ok(packets)
    }
}

fn checkpoint_onsets(samples: &[f32], profile: PcmProfile) -> Vec<usize> {
    const ONSET_THRESHOLD: f32 = 0.02;
    const QUIET_RUN_MS: u32 = 10;

    let required_quiet = profile.samples_for_ms(QUIET_RUN_MS).max(1);
    let mut quiet_samples = required_quiet;
    let mut onsets = Vec::new();
    for (index, sample) in samples.iter().enumerate() {
        if sample.abs() < ONSET_THRESHOLD {
            quiet_samples = quiet_samples.saturating_add(1);
        } else {
            if quiet_samples >= required_quiet {
                onsets.push(index.saturating_sub(1));
            }
            quiet_samples = 0;
        }
    }
    onsets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_skips_an_incomplete_frame() {
        let profile = PcmProfile::with_timing_percent(200).unwrap();
        let codec = FixedPcmCodec::new(profile);
        let incomplete = codec.encode(b"incomplete").unwrap();
        let complete = codec.encode(b"complete").unwrap();
        let mut recording = incomplete[incomplete.len() / 2..].to_vec();
        recording.extend(complete);
        assert_eq!(
            codec.decode_all_at_sample_rate(&recording, 8_000).unwrap(),
            vec![b"complete".to_vec()]
        );
    }
}
