use super::decode::{find_onset, goertzel, invalid_audio, slice};
use super::synth::{
    append_silence, append_tones, midi_frequency, AFTER_HEADER_MS, AFTER_SYNC_MS, HEADER_BIT_MS,
    HEADER_ONE_NOTE, HEADER_ZERO_NOTE, LEADING_SILENCE_MS, SYNC_GAP_MS, SYNC_HIGH_NOTE,
    SYNC_LOW_NOTE, SYNC_TONE_MS,
};
use super::{resample, PcmProfile, MAX_PCM_PACKET_BYTES, MAX_PCM_SAMPLES};
use crate::error::LogiscoreError;

const SYMBOL_TONE_MS: u32 = 80;
const SYMBOL_REST_MS: u32 = 40;
const FIRST_SYMBOL_NOTE: u8 = 60;

#[derive(Debug, Clone, Copy)]
pub struct FixedPcmCodec {
    profile: PcmProfile,
}

impl FixedPcmCodec {
    pub const fn new(profile: PcmProfile) -> Self {
        Self { profile }
    }

    pub fn encode(self, packet: &[u8]) -> Result<Vec<f32>, LogiscoreError> {
        self.validate_packet(packet)?;
        let capacity = self.estimated_samples(packet.len())?;
        let mut samples = Vec::with_capacity(capacity);
        append_silence(
            &mut samples,
            self.profile.samples_for_ms(LEADING_SILENCE_MS),
        );
        self.append_framing(&mut samples, packet.len());
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

    pub fn decode_at_sample_rate(
        self,
        samples: &[f32],
        source_sample_rate: u32,
    ) -> Result<Vec<u8>, LogiscoreError> {
        let normalized = resample::linear(
            samples,
            source_sample_rate,
            self.profile.sample_rate(),
            MAX_PCM_SAMPLES,
        )?;
        self.decode(&normalized)
    }

    pub fn decode(self, samples: &[f32]) -> Result<Vec<u8>, LogiscoreError> {
        if samples.len() > MAX_PCM_SAMPLES || samples.iter().any(|sample| !sample.is_finite()) {
            return Err(invalid_audio("fixed PCM input is invalid or too large"));
        }
        let sync_start =
            find_onset(samples, 0).ok_or_else(|| invalid_audio("preamble not found"))?;
        self.verify_sync(samples, sync_start)?;
        let header_start = sync_start
            + self
                .profile
                .samples_for_ms(SYNC_TONE_MS * 2 + SYNC_GAP_MS + AFTER_SYNC_MS);
        let packet_length = self.decode_length(samples, header_start)?;
        let bit_samples = self.profile.samples_for_ms(HEADER_BIT_MS);
        let mut cursor = header_start
            .checked_add(bit_samples.saturating_mul(32))
            .and_then(|value| value.checked_add(self.profile.samples_for_ms(AFTER_HEADER_MS)))
            .ok_or_else(|| invalid_audio("fixed PCM cursor overflow"))?;
        let tone_samples = self.profile.samples_for_ms(SYMBOL_TONE_MS);
        let stride = tone_samples + self.profile.samples_for_ms(SYMBOL_REST_MS);
        let mut packet = Vec::with_capacity(packet_length);
        for _ in 0..packet_length {
            let high = self.decode_nibble(slice(samples, cursor, tone_samples)?)?;
            cursor = cursor.saturating_add(stride);
            let low = self.decode_nibble(slice(samples, cursor, tone_samples)?)?;
            cursor = cursor.saturating_add(stride);
            packet.push((high << 4) | low);
        }
        Ok(packet)
    }

    fn append_framing(self, samples: &mut Vec<f32>, packet_length: usize) {
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
        for bit in (0..32).rev() {
            let note = if (packet_length as u32 >> bit) & 1 == 0 {
                HEADER_ZERO_NOTE
            } else {
                HEADER_ONE_NOTE
            };
            append_tones(
                samples,
                &[note],
                self.profile.samples_for_ms(HEADER_BIT_MS),
                self.profile.sample_rate(),
                0.6,
            );
        }
        append_silence(samples, self.profile.samples_for_ms(AFTER_HEADER_MS));
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
        if expected_energy < 0.01 || expected_energy < alternate_energy * 2.0 {
            return Err(invalid_audio("invalid fixed synchronization tone"));
        }
        Ok(())
    }

    fn decode_length(self, samples: &[f32], start: usize) -> Result<usize, LogiscoreError> {
        let count = self.profile.samples_for_ms(HEADER_BIT_MS);
        let mut length = 0u32;
        for bit in 0..32 {
            let window = slice(samples, start + bit * count, count)?;
            let zero = goertzel(
                window,
                midi_frequency(HEADER_ZERO_NOTE),
                self.profile.sample_rate(),
            );
            let one = goertzel(
                window,
                midi_frequency(HEADER_ONE_NOTE),
                self.profile.sample_rate(),
            );
            length = (length << 1) | u32::from(one > zero);
        }
        let length =
            usize::try_from(length).map_err(|_| invalid_audio("fixed packet length overflow"))?;
        if length > MAX_PCM_PACKET_BYTES {
            return Err(invalid_audio("fixed packet length exceeds limit"));
        }
        Ok(length)
    }

    fn decode_nibble(self, samples: &[f32]) -> Result<u8, LogiscoreError> {
        (0..16u8)
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
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .filter(|(energy, _)| *energy > 0.01)
            .map(|(_, nibble)| nibble)
            .ok_or_else(|| invalid_audio("fixed symbol was not detected"))
    }

    fn validate_packet(self, packet: &[u8]) -> Result<(), LogiscoreError> {
        if packet.len() > MAX_PCM_PACKET_BYTES {
            return Err(invalid_audio("fixed packet exceeds limit"));
        }
        if self.estimated_samples(packet.len())? > MAX_PCM_SAMPLES {
            return Err(invalid_audio("fixed PCM output exceeds sample limit"));
        }
        Ok(())
    }

    fn estimated_samples(self, packet_length: usize) -> Result<usize, LogiscoreError> {
        let framing_ms = LEADING_SILENCE_MS
            + SYNC_TONE_MS * 2
            + SYNC_GAP_MS
            + AFTER_SYNC_MS
            + HEADER_BIT_MS * 32
            + AFTER_HEADER_MS;
        let framing = self.profile.samples_for_ms(framing_ms);
        let symbol = self.profile.samples_for_ms(SYMBOL_TONE_MS + SYMBOL_REST_MS);
        framing
            .checked_add(packet_length.saturating_mul(2).saturating_mul(symbol))
            .ok_or_else(|| invalid_audio("fixed PCM size overflow"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_codec_roundtrips_bytes_with_leading_offset() {
        let profile = PcmProfile::with_timing_percent(200).unwrap();
        let codec = FixedPcmCodec::new(profile);
        let mut samples = vec![0.0; 317];
        samples.extend(codec.encode(b"fixed fallback").unwrap());
        assert_eq!(codec.decode(&samples).unwrap(), b"fixed fallback");
    }
}
