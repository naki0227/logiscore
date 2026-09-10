use super::synth::{
    midi_frequency, AFTER_HEADER_MS, AFTER_SYNC_MS, HEADER_BIT_MS, HEADER_ONE_NOTE,
    HEADER_ZERO_NOTE, SYNC_GAP_MS, SYNC_HIGH_NOTE, SYNC_LOW_NOTE, SYNC_TONE_MS,
};
use super::{PcmCodec, MAX_PCM_PACKET_BYTES, MAX_PCM_SAMPLES};
use crate::error::LogiscoreError;
use crate::musical::{CandidateSet, RhythmicEvent, RhythmicMusicalCodec};

const ONSET_THRESHOLD: f32 = 0.02;
const SILENCE_THRESHOLD: f32 = 0.01;

pub(crate) fn decode(codec: &PcmCodec, samples: &[f32]) -> Result<Vec<u8>, LogiscoreError> {
    if samples.len() > MAX_PCM_SAMPLES {
        return Err(invalid_audio("PCM input exceeds the decoder limit"));
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(invalid_audio("PCM contains non-finite samples"));
    }
    let sync_start = find_onset(samples, 0).ok_or_else(|| invalid_audio("preamble not found"))?;
    let sync_samples = codec.profile.samples_for_ms(SYNC_TONE_MS);
    verify_tone(codec, samples, sync_start, sync_samples, SYNC_HIGH_NOTE)?;
    let low_start = sync_start + sync_samples + codec.profile.samples_for_ms(SYNC_GAP_MS);
    verify_tone(codec, samples, low_start, sync_samples, SYNC_LOW_NOTE)?;
    let header_start = low_start + sync_samples + codec.profile.samples_for_ms(AFTER_SYNC_MS);
    let bit_samples = codec.profile.samples_for_ms(HEADER_BIT_MS);
    let packet_length = decode_length(codec, samples, header_start, bit_samples)?;
    let mut cursor = header_start
        .checked_add(bit_samples * 32)
        .and_then(|position| position.checked_add(codec.profile.samples_for_ms(AFTER_HEADER_MS)))
        .ok_or_else(|| invalid_audio("PCM cursor overflow"))?;
    let event_count = RhythmicMusicalCodec::event_count_for_bytes(packet_length);
    let mut previous = None;
    let mut events = Vec::with_capacity(event_count);
    for event_index in 0..event_count {
        let start =
            find_onset(samples, cursor).ok_or_else(|| invalid_audio("missing rhythmic event"))?;
        let end = find_tone_end(samples, start, codec.profile.rest_samples() / 4)
            .ok_or_else(|| invalid_audio("unterminated rhythmic event"))?;
        let duration_ticks = detect_duration(codec, end - start)?;
        let candidates = CandidateSet::generate_with_count(
            codec.planner.chord_at(event_index),
            previous,
            RhythmicMusicalCodec::pitch_candidate_count(event_index),
        )
        .map_err(|error| invalid_audio(&error.to_string()))?;
        let voicing = detect_voicing(codec, &samples[start..end], &candidates)?;
        events.push(RhythmicEvent::from_parts(voicing, duration_ticks));
        previous = Some(voicing);
        cursor = end.saturating_add(codec.profile.rest_samples() / 2);
    }
    codec
        .rhythmic
        .decode(&events, packet_length)
        .map_err(|error| invalid_audio(&error.to_string()))
}

fn decode_length(
    codec: &PcmCodec,
    samples: &[f32],
    start: usize,
    bit_samples: usize,
) -> Result<usize, LogiscoreError> {
    let mut length = 0u32;
    for bit in 0..32 {
        let offset = start + bit * bit_samples;
        let window = slice(samples, offset, bit_samples)?;
        let zero = goertzel(
            window,
            midi_frequency(HEADER_ZERO_NOTE),
            codec.sample_rate(),
        );
        let one = goertzel(window, midi_frequency(HEADER_ONE_NOTE), codec.sample_rate());
        length = (length << 1) | u32::from(one > zero);
    }
    let length = usize::try_from(length).map_err(|_| invalid_audio("packet length overflow"))?;
    if length > MAX_PCM_PACKET_BYTES {
        return Err(invalid_audio("packet length exceeds PCM profile limit"));
    }
    Ok(length)
}

fn verify_tone(
    codec: &PcmCodec,
    samples: &[f32],
    start: usize,
    sample_count: usize,
    note: u8,
) -> Result<(), LogiscoreError> {
    let window = slice(samples, start, sample_count)?;
    let expected = goertzel(window, midi_frequency(note), codec.sample_rate());
    let alternate_note = if note == SYNC_HIGH_NOTE {
        SYNC_LOW_NOTE
    } else {
        SYNC_HIGH_NOTE
    };
    let alternate = goertzel(window, midi_frequency(alternate_note), codec.sample_rate());
    if expected < 0.01 || expected < alternate * 2.0 {
        return Err(invalid_audio("invalid synchronization tone"));
    }
    Ok(())
}

fn detect_duration(codec: &PcmCodec, measured_samples: usize) -> Result<u16, LogiscoreError> {
    [240u16, 360, 480, 720]
        .into_iter()
        .min_by_key(|ticks| {
            codec
                .profile
                .duration_samples(*ticks)
                .abs_diff(measured_samples)
        })
        .filter(|ticks| {
            codec
                .profile
                .duration_samples(*ticks)
                .abs_diff(measured_samples)
                <= codec.profile.samples_for_ms(12)
        })
        .ok_or_else(|| invalid_audio("unknown rhythmic duration"))
}

fn detect_voicing(
    codec: &PcmCodec,
    samples: &[f32],
    candidates: &CandidateSet,
) -> Result<crate::musical::Voicing, LogiscoreError> {
    candidates
        .candidates()
        .iter()
        .copied()
        .map(|voicing| {
            let energy = voicing
                .notes()
                .into_iter()
                .map(|note| goertzel(samples, midi_frequency(note), codec.sample_rate()))
                .sum::<f32>();
            (energy, voicing)
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))
        .filter(|(energy, _)| *energy > 0.001)
        .map(|(_, voicing)| voicing)
        .ok_or_else(|| invalid_audio("musical chord was not detected"))
}

pub(super) fn goertzel(samples: &[f32], frequency: f32, sample_rate: u32) -> f32 {
    let coefficient = 2.0 * (std::f32::consts::TAU * frequency / sample_rate as f32).cos();
    let mut previous = 0.0;
    let mut before_previous = 0.0;
    for &sample in samples {
        let current = sample + coefficient * previous - before_previous;
        before_previous = previous;
        previous = current;
    }
    let power = previous * previous + before_previous * before_previous
        - coefficient * previous * before_previous;
    power / (samples.len().max(1) as f32).powi(2)
}

pub(super) fn find_onset(samples: &[f32], start: usize) -> Option<usize> {
    samples
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, sample)| sample.abs() >= ONSET_THRESHOLD)
        .map(|(index, _)| index.saturating_sub(1))
}

fn find_tone_end(samples: &[f32], start: usize, silence_run: usize) -> Option<usize> {
    let mut quiet = 0;
    for (index, sample) in samples.iter().enumerate().skip(start) {
        if sample.abs() < SILENCE_THRESHOLD {
            quiet += 1;
            if quiet >= silence_run {
                return Some(index + 1 - quiet);
            }
        } else {
            quiet = 0;
        }
    }
    None
}

pub(super) fn slice(
    samples: &[f32],
    start: usize,
    length: usize,
) -> Result<&[f32], LogiscoreError> {
    start
        .checked_add(length)
        .filter(|end| *end <= samples.len())
        .map(|end| &samples[start..end])
        .ok_or_else(|| invalid_audio("PCM data is truncated"))
}

pub(super) fn invalid_audio(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidAudio(message.to_owned())
}
