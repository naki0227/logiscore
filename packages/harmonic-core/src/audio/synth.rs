use std::f32::consts::TAU;

use super::PcmCodec;
use crate::error::LogiscoreError;
use crate::musical::RhythmicMusicalCodec;

pub(crate) const SYNC_HIGH_NOTE: u8 = 96;
pub(crate) const SYNC_LOW_NOTE: u8 = 84;
pub(crate) const HEADER_ZERO_NOTE: u8 = 88;
pub(crate) const HEADER_ONE_NOTE: u8 = 92;
pub(crate) const LEADING_SILENCE_MS: u32 = 80;
pub(crate) const SYNC_TONE_MS: u32 = 160;
pub(crate) const SYNC_GAP_MS: u32 = 40;
pub(crate) const AFTER_SYNC_MS: u32 = 80;
pub(crate) const HEADER_BIT_MS: u32 = 40;
pub(crate) const AFTER_HEADER_MS: u32 = 80;

pub(crate) fn estimated_max_samples(
    codec: &PcmCodec,
    packet_length: usize,
) -> Result<usize, LogiscoreError> {
    let framing_ms = LEADING_SILENCE_MS
        + SYNC_TONE_MS * 2
        + SYNC_GAP_MS
        + AFTER_SYNC_MS
        + HEADER_BIT_MS * 32
        + AFTER_HEADER_MS;
    let framing = codec.profile.samples_for_ms(framing_ms);
    let event_count = RhythmicMusicalCodec::event_count_for_bytes(packet_length);
    let max_event = codec
        .profile
        .duration_samples(720)
        .checked_add(codec.profile.rest_samples())
        .ok_or_else(|| LogiscoreError::InvalidAudio("PCM size overflow".into()))?;
    framing
        .checked_add(event_count.saturating_mul(max_event))
        .ok_or_else(|| LogiscoreError::InvalidAudio("PCM size overflow".into()))
}

pub(crate) fn encode(codec: &PcmCodec, packet: &[u8]) -> Result<Vec<f32>, LogiscoreError> {
    let events = codec
        .rhythmic
        .encode(packet)
        .map_err(|error| LogiscoreError::InvalidAudio(error.to_string()))?;
    let mut samples = Vec::new();
    append_silence(
        &mut samples,
        codec.profile.samples_for_ms(LEADING_SILENCE_MS),
    );
    append_tones(
        &mut samples,
        &[SYNC_HIGH_NOTE],
        codec.profile.samples_for_ms(SYNC_TONE_MS),
        codec.sample_rate(),
        0.6,
    );
    append_silence(&mut samples, codec.profile.samples_for_ms(SYNC_GAP_MS));
    append_tones(
        &mut samples,
        &[SYNC_LOW_NOTE],
        codec.profile.samples_for_ms(SYNC_TONE_MS),
        codec.sample_rate(),
        0.6,
    );
    append_silence(&mut samples, codec.profile.samples_for_ms(AFTER_SYNC_MS));
    for bit in (0..32).rev() {
        let note = if (packet.len() as u32 >> bit) & 1 == 0 {
            HEADER_ZERO_NOTE
        } else {
            HEADER_ONE_NOTE
        };
        append_tones(
            &mut samples,
            &[note],
            codec.profile.samples_for_ms(HEADER_BIT_MS),
            codec.sample_rate(),
            0.6,
        );
    }
    append_silence(&mut samples, codec.profile.samples_for_ms(AFTER_HEADER_MS));
    for event in events {
        append_tones(
            &mut samples,
            &event.voicing().notes(),
            codec.profile.duration_samples(event.duration_ticks()),
            codec.sample_rate(),
            0.18,
        );
        append_silence(&mut samples, codec.profile.rest_samples());
    }
    Ok(samples)
}

pub(crate) fn midi_frequency(note: u8) -> f32 {
    440.0 * 2.0f32.powf((f32::from(note) - 69.0) / 12.0)
}

pub(super) fn append_tones(
    output: &mut Vec<f32>,
    notes: &[u8],
    sample_count: usize,
    sample_rate: u32,
    amplitude: f32,
) {
    for index in 0..sample_count {
        let time = index as f32 / sample_rate as f32;
        let sample = notes
            .iter()
            .map(|note| (TAU * midi_frequency(*note) * time).sin())
            .sum::<f32>()
            * amplitude;
        output.push(sample);
    }
}

pub(super) fn append_silence(output: &mut Vec<f32>, sample_count: usize) {
    output.resize(output.len() + sample_count, 0.0);
}
