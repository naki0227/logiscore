use crate::error::LogiscoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmProfile {
    sample_rate: u32,
    timing_percent: u16,
}

impl Default for PcmProfile {
    fn default() -> Self {
        Self {
            sample_rate: 8_000,
            timing_percent: 100,
        }
    }
}

impl PcmProfile {
    pub const fn sample_rate(self) -> u32 {
        self.sample_rate
    }

    pub const fn timing_percent(self) -> u16 {
        self.timing_percent
    }

    pub fn with_timing_percent(timing_percent: u16) -> Result<Self, LogiscoreError> {
        if !(50..=250).contains(&timing_percent) {
            return Err(LogiscoreError::InvalidProfile(
                "PCM timing must be between 50% and 250%".into(),
            ));
        }
        Ok(Self {
            timing_percent,
            ..Self::default()
        })
    }

    pub(crate) fn samples_for_ms(self, milliseconds: u32) -> usize {
        let samples = u64::from(self.sample_rate)
            .saturating_mul(u64::from(milliseconds))
            .saturating_mul(u64::from(self.timing_percent))
            / 100_000;
        usize::try_from(samples).unwrap_or(usize::MAX)
    }

    pub(crate) fn duration_samples(self, ticks: u16) -> usize {
        self.samples_for_ms(u32::from(ticks) / 3)
    }

    pub(crate) fn rest_samples(self) -> usize {
        self.samples_for_ms(40)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_scales_all_pcm_intervals() {
        let quick = PcmProfile::with_timing_percent(75).unwrap();
        let robust = PcmProfile::with_timing_percent(175).unwrap();
        assert_eq!(quick.samples_for_ms(40), 240);
        assert_eq!(robust.samples_for_ms(40), 560);
    }

    #[test]
    fn timing_rejects_unsupported_values() {
        assert!(PcmProfile::with_timing_percent(49).is_err());
        assert!(PcmProfile::with_timing_percent(251).is_err());
    }
}
