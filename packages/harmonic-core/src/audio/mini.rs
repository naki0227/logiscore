use super::decode::{goertzel, invalid_audio, slice};
use super::fixed::validate_samples;
use super::fixed_scan::checkpoint_onsets;
use super::synth::{append_silence, append_tones, midi_frequency};
use super::{resample, PcmProfile, MAX_PCM_SAMPLES};
use crate::error::LogiscoreError;

const LEADING_SILENCE_MS: u32 = 20;
const SYNC_TONE_MS: u32 = 60;
const SYNC_GAP_MS: u32 = 20;
const AFTER_SYNC_MS: u32 = 20;
const LENGTH_BIT_MS: u32 = 20;
const LENGTH_BITS: usize = 10;
const AFTER_LENGTH_MS: u32 = 20;
const SYMBOL_TONE_MS: u32 = 80;
const SYMBOL_REST_MS: u32 = 40;
const SYNC_HIGH_NOTE: u8 = 96;
const SYNC_LOW_NOTE: u8 = 84;
const LENGTH_ZERO_NOTE: u8 = 88;
const LENGTH_ONE_NOTE: u8 = 92;
const FIRST_SYMBOL_NOTE: u8 = 60;
const MIN_SYNC_TONE_ENERGY: f32 = 0.0001;
const MIN_SYMBOL_TONE_ENERGY: f32 = 0.00000001;
const MAX_MINI_PACKET_BYTES: usize = (1 << LENGTH_BITS) - 1;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MiniFixedPcmCodec {
    profile: PcmProfile,
}

#[derive(Debug, Clone)]
pub(crate) struct MiniPcmObservation {
    pub(crate) bytes: Vec<u8>,
    pub(crate) nibble_confidences: Vec<f32>,
}

impl MiniFixedPcmCodec {
    pub const fn new(profile: PcmProfile) -> Self {
        Self { profile }
    }

    pub fn encode(self, packet: &[u8]) -> Result<Vec<f32>, LogiscoreError> {
        if packet.len() > MAX_MINI_PACKET_BYTES {
            return Err(invalid_audio("mini PCM packet exceeds 10-bit length limit"));
        }
        let mut samples = Vec::with_capacity(self.estimated_samples(packet.len())?);
        append_silence(
            &mut samples,
            self.profile.samples_for_ms(LEADING_SILENCE_MS),
        );
        self.append_sync(&mut samples);
        self.append_length(&mut samples, packet.len());
        for nibble in packet.iter().flat_map(|byte| [byte >> 4, byte & 0x0f]) {
            append_tones(
                &mut samples,
                &[FIRST_SYMBOL_NOTE + nibble],
                self.profile.samples_for_ms(SYMBOL_TONE_MS),
                self.profile.sample_rate(),
                0.6,
            );
            append_silence(&mut samples, self.profile.samples_for_ms(SYMBOL_REST_MS));
        }
        Ok(samples)
    }

    pub fn decode_all_observations_at_sample_rate(
        self,
        samples: &[f32],
        source_sample_rate: u32,
    ) -> Result<Vec<MiniPcmObservation>, LogiscoreError> {
        let normalized = resample::linear(
            samples,
            source_sample_rate,
            self.profile.sample_rate(),
            MAX_PCM_SAMPLES,
        )?;
        validate_samples(&normalized)?;
        let mut observations = Vec::new();
        let mut consumed_until = 0;
        for sync_start in checkpoint_onsets(&normalized, self.profile) {
            if sync_start < consumed_until {
                continue;
            }
            if let Ok((observation, frame_end)) = self.decode_from_sync(&normalized, sync_start) {
                observations.push(observation);
                consumed_until = frame_end;
            }
        }
        Ok(observations)
    }

    fn decode_from_sync(
        self,
        samples: &[f32],
        sync_start: usize,
    ) -> Result<(MiniPcmObservation, usize), LogiscoreError> {
        self.verify_sync(samples, sync_start)?;
        let length_start = sync_start
            + self
                .profile
                .samples_for_ms(SYNC_TONE_MS * 2 + SYNC_GAP_MS + AFTER_SYNC_MS);
        let packet_length = self.decode_length(samples, length_start)?;
        let mut cursor = length_start
            .checked_add(
                self.profile
                    .samples_for_ms(LENGTH_BIT_MS)
                    .saturating_mul(LENGTH_BITS),
            )
            .and_then(|value| value.checked_add(self.profile.samples_for_ms(AFTER_LENGTH_MS)))
            .ok_or_else(|| invalid_audio("mini PCM cursor overflow"))?;
        let tone_samples = self.profile.samples_for_ms(SYMBOL_TONE_MS);
        let stride = tone_samples + self.profile.samples_for_ms(SYMBOL_REST_MS);
        let mut packet = Vec::with_capacity(packet_length);
        let mut nibble_confidences = Vec::with_capacity(packet_length.saturating_mul(2));
        for _ in 0..packet_length {
            let (high, high_confidence) =
                self.decode_nibble(slice(samples, cursor, tone_samples)?)?;
            cursor = cursor.saturating_add(stride);
            let (low, low_confidence) =
                self.decode_nibble(slice(samples, cursor, tone_samples)?)?;
            cursor = cursor.saturating_add(stride);
            packet.push((high << 4) | low);
            nibble_confidences.extend([high_confidence, low_confidence]);
        }
        Ok((
            MiniPcmObservation {
                bytes: packet,
                nibble_confidences,
            },
            cursor,
        ))
    }

    fn append_sync(self, samples: &mut Vec<f32>) {
        append_tones(
            samples,
            &[SYNC_HIGH_NOTE],
            self.profile.samples_for_ms(SYNC_TONE_MS),
            self.profile.sample_rate(),
            0.6,
        );
        append_silence(samples, self.profile.samples_for_ms(SYNC_GAP_MS));
        append_tones(
            samples,
            &[SYNC_LOW_NOTE],
            self.profile.samples_for_ms(SYNC_TONE_MS),
            self.profile.sample_rate(),
            0.6,
        );
        append_silence(samples, self.profile.samples_for_ms(AFTER_SYNC_MS));
    }

    fn append_length(self, samples: &mut Vec<f32>, packet_length: usize) {
        for bit in (0..LENGTH_BITS).rev() {
            let note = if (packet_length >> bit) & 1 == 0 {
                LENGTH_ZERO_NOTE
            } else {
                LENGTH_ONE_NOTE
            };
            append_tones(
                samples,
                &[note],
                self.profile.samples_for_ms(LENGTH_BIT_MS),
                self.profile.sample_rate(),
                0.6,
            );
        }
        append_silence(samples, self.profile.samples_for_ms(AFTER_LENGTH_MS));
    }

    fn verify_sync(self, samples: &[f32], start: usize) -> Result<(), LogiscoreError> {
        let count = self.profile.samples_for_ms(SYNC_TONE_MS);
        self.verify_note(slice(samples, start, count)?, SYNC_HIGH_NOTE, SYNC_LOW_NOTE)?;
        let low_start = start + count + self.profile.samples_for_ms(SYNC_GAP_MS);
        self.verify_note(
            slice(samples, low_start, count)?,
            SYNC_LOW_NOTE,
            SYNC_HIGH_NOTE,
        )
    }

    fn verify_note(
        self,
        samples: &[f32],
        expected: u8,
        alternate: u8,
    ) -> Result<(), LogiscoreError> {
        let expected_energy = goertzel(
            samples,
            midi_frequency(expected),
            self.profile.sample_rate(),
        );
        let alternate_energy = goertzel(
            samples,
            midi_frequency(alternate),
            self.profile.sample_rate(),
        );
        if expected_energy < MIN_SYNC_TONE_ENERGY || expected_energy < alternate_energy * 2.0 {
            return Err(invalid_audio("invalid mini synchronization tone"));
        }
        Ok(())
    }

    fn decode_length(self, samples: &[f32], start: usize) -> Result<usize, LogiscoreError> {
        let count = self.profile.samples_for_ms(LENGTH_BIT_MS);
        let mut length = 0usize;
        for bit in 0..LENGTH_BITS {
            let window = slice(samples, start + bit * count, count)?;
            let zero = goertzel(
                window,
                midi_frequency(LENGTH_ZERO_NOTE),
                self.profile.sample_rate(),
            );
            let one = goertzel(
                window,
                midi_frequency(LENGTH_ONE_NOTE),
                self.profile.sample_rate(),
            );
            length = (length << 1) | usize::from(one > zero);
        }
        Ok(length)
    }

    fn decode_nibble(self, samples: &[f32]) -> Result<(u8, f32), LogiscoreError> {
        let mut energies = (0..16u8)
            .map(|nibble| {
                (
                    goertzel(
                        samples,
                        midi_frequency(FIRST_SYMBOL_NOTE + nibble),
                        self.profile.sample_rate(),
                    ),
                    nibble,
                )
            })
            .collect::<Vec<_>>();
        energies.sort_by(|left, right| right.0.total_cmp(&left.0));
        let (best_energy, nibble) = energies[0];
        if best_energy <= MIN_SYMBOL_TONE_ENERGY {
            return Err(invalid_audio("mini symbol was not detected"));
        }
        let second_energy = energies[1].0;
        let confidence =
            ((best_energy - second_energy) / best_energy.max(f32::EPSILON)).clamp(0.0, 1.0);
        Ok((nibble, confidence))
    }

    fn estimated_samples(self, packet_length: usize) -> Result<usize, LogiscoreError> {
        let framing_ms = LEADING_SILENCE_MS
            + SYNC_TONE_MS * 2
            + SYNC_GAP_MS
            + AFTER_SYNC_MS
            + LENGTH_BIT_MS * LENGTH_BITS as u32
            + AFTER_LENGTH_MS;
        self.profile
            .samples_for_ms(framing_ms)
            .checked_add(
                packet_length
                    .saturating_mul(2)
                    .saturating_mul(self.profile.samples_for_ms(SYMBOL_TONE_MS + SYMBOL_REST_MS)),
            )
            .filter(|length| *length <= MAX_PCM_SAMPLES)
            .ok_or_else(|| invalid_audio("mini PCM output exceeds sample limit"))
    }

    #[cfg(test)]
    pub(crate) fn overwrite_encoded_nibble(
        self,
        samples: &mut [f32],
        byte_index: usize,
        high_nibble: bool,
        replacement: u8,
    ) {
        let framing_ms = LEADING_SILENCE_MS
            + SYNC_TONE_MS * 2
            + SYNC_GAP_MS
            + AFTER_SYNC_MS
            + LENGTH_BIT_MS * LENGTH_BITS as u32
            + AFTER_LENGTH_MS;
        let stride = self.profile.samples_for_ms(SYMBOL_TONE_MS + SYMBOL_REST_MS);
        let nibble_index = byte_index * 2 + usize::from(!high_nibble);
        let start = self.profile.samples_for_ms(framing_ms) + nibble_index * stride;
        let tone_samples = self.profile.samples_for_ms(SYMBOL_TONE_MS);
        let frequency = midi_frequency(FIRST_SYMBOL_NOTE + replacement);
        for (index, sample) in samples[start..start + tone_samples].iter_mut().enumerate() {
            let time = index as f32 / self.profile.sample_rate() as f32;
            *sample = (std::f32::consts::TAU * frequency * time).sin() * 0.6;
        }
    }
}

#[cfg(test)]
mod tests;
