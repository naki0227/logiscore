mod channel;
mod decode;
mod fixed;
mod fixed_scan;
mod mini;
mod profile;
mod resample;
mod synth;
mod wav;

pub use channel::ChannelModel;
pub use fixed::FixedPcmCodec;
pub(crate) use mini::{MiniFixedPcmCodec, MiniPcmObservation};
pub use profile::PcmProfile;
pub use wav::{decode_wav, encode_wav, PcmAudio};

use crate::error::LogiscoreError;
use crate::musical::{ChordPlanner, RhythmicMusicalCodec, TonalContext};

pub(crate) const MAX_PCM_PACKET_BYTES: usize = 4 * 1024;
pub(crate) const MAX_PCM_SAMPLES: usize = 16_000_000;

#[derive(Debug, Clone, Copy)]
pub struct PcmCodec {
    profile: PcmProfile,
    rhythmic: RhythmicMusicalCodec,
    planner: ChordPlanner,
}

impl Default for PcmCodec {
    fn default() -> Self {
        Self {
            profile: PcmProfile::default(),
            rhythmic: RhythmicMusicalCodec::new(TonalContext::C_MAJOR),
            planner: ChordPlanner::new(TonalContext::C_MAJOR),
        }
    }
}

impl PcmCodec {
    pub fn with_profile(profile: PcmProfile) -> Self {
        Self {
            profile,
            ..Self::default()
        }
    }

    pub const fn sample_rate(self) -> u32 {
        self.profile.sample_rate()
    }

    pub fn encode(&self, packet: &[u8]) -> Result<Vec<f32>, LogiscoreError> {
        if packet.len() > MAX_PCM_PACKET_BYTES {
            return Err(LogiscoreError::InvalidAudio(format!(
                "PCM packet exceeds {MAX_PCM_PACKET_BYTES} bytes"
            )));
        }
        let estimated_samples = synth::estimated_max_samples(self, packet.len())?;
        if estimated_samples > MAX_PCM_SAMPLES {
            return Err(LogiscoreError::InvalidAudio(format!(
                "PCM output exceeds {MAX_PCM_SAMPLES} samples for the selected profile"
            )));
        }
        synth::encode(self, packet)
    }

    pub fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, LogiscoreError> {
        decode::decode(self, samples)
    }

    pub fn decode_at_sample_rate(
        &self,
        samples: &[f32],
        source_sample_rate: u32,
    ) -> Result<Vec<u8>, LogiscoreError> {
        let normalized = resample::linear(
            samples,
            source_sample_rate,
            self.sample_rate(),
            MAX_PCM_SAMPLES,
        )?;
        self.decode(&normalized)
    }
}

#[cfg(test)]
mod tests;
