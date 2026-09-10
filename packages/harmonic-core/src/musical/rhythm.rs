use super::{CandidateSet, ChordPlanner, MusicalError, TonalContext, Voicing};

const RHYTHMS: [u16; 4] = [240, 360, 480, 720];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RhythmicEvent {
    voicing: Voicing,
    duration_ticks: u16,
}

impl RhythmicEvent {
    pub(crate) const fn from_parts(voicing: Voicing, duration_ticks: u16) -> Self {
        Self {
            voicing,
            duration_ticks,
        }
    }

    pub const fn voicing(self) -> Voicing {
        self.voicing
    }

    pub const fn duration_ticks(self) -> u16 {
        self.duration_ticks
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RhythmicMusicalCodec {
    planner: ChordPlanner,
}

impl RhythmicMusicalCodec {
    pub const fn new(context: TonalContext) -> Self {
        Self {
            planner: ChordPlanner::new(context),
        }
    }

    pub fn encode(&self, bytes: &[u8]) -> Result<Vec<RhythmicEvent>, MusicalError> {
        let bit_length = bytes.len() * 8;
        let mut bit_offset = 0;
        let mut event_index = 0;
        let mut previous = None;
        let mut events = Vec::with_capacity(events_for_bits(bit_length));
        while bit_offset < bit_length {
            let pitch_bits = pitch_bits_at(event_index);
            let pitch_symbol = read_bits(bytes, &mut bit_offset, pitch_bits);
            let rhythm_symbol = read_bits(bytes, &mut bit_offset, 2);
            let candidates = CandidateSet::generate_with_count(
                self.planner.chord_at(event_index),
                previous,
                1 << pitch_bits,
            )?;
            let voicing = candidates.encode_symbol(pitch_symbol)?;
            events.push(RhythmicEvent {
                voicing,
                duration_ticks: RHYTHMS[usize::from(rhythm_symbol)],
            });
            previous = Some(voicing);
            event_index += 1;
        }
        Ok(events)
    }

    pub fn decode(
        &self,
        events: &[RhythmicEvent],
        expected_bytes: usize,
    ) -> Result<Vec<u8>, MusicalError> {
        let expected_bits = expected_bytes * 8;
        if events.len() != events_for_bits(expected_bits) {
            return Err(MusicalError::UnknownVoicing);
        }
        let mut bits = Vec::new();
        let mut previous = None;
        for (event_index, event) in events.iter().enumerate() {
            let pitch_bits = pitch_bits_at(event_index);
            let candidates = CandidateSet::generate_with_count(
                self.planner.chord_at(event_index),
                previous,
                1 << pitch_bits,
            )?;
            let pitch_symbol = candidates.decode_symbol(event.voicing)?;
            let rhythm_symbol = RHYTHMS
                .iter()
                .position(|duration| *duration == event.duration_ticks)
                .ok_or(MusicalError::UnknownVoicing)? as u8;
            push_bits(&mut bits, pitch_symbol, pitch_bits);
            push_bits(&mut bits, rhythm_symbol, 2);
            previous = Some(event.voicing);
        }
        if bits[expected_bits..].iter().any(|bit| *bit) {
            return Err(MusicalError::UnknownVoicing);
        }
        bits.truncate(expected_bits);
        Ok(bits
            .chunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0, |byte, bit| (byte << 1) | u8::from(*bit))
            })
            .collect())
    }

    pub fn event_capacity_bits(event_index: usize) -> usize {
        pitch_bits_at(event_index) + 2
    }

    pub fn event_count_for_bytes(byte_length: usize) -> usize {
        events_for_bits(byte_length * 8)
    }

    pub fn pitch_candidate_count(event_index: usize) -> usize {
        1 << pitch_bits_at(event_index)
    }
}

fn pitch_bits_at(event_index: usize) -> usize {
    if event_index.is_multiple_of(2) {
        3
    } else {
        5
    }
}

fn events_for_bits(bit_length: usize) -> usize {
    let mut covered = 0;
    let mut events = 0;
    while covered < bit_length {
        covered += RhythmicMusicalCodec::event_capacity_bits(events);
        events += 1;
    }
    events
}

fn read_bits(bytes: &[u8], offset: &mut usize, count: usize) -> u8 {
    let mut value = 0;
    for _ in 0..count {
        value <<= 1;
        if *offset < bytes.len() * 8 {
            let byte = bytes[*offset / 8];
            value |= (byte >> (7 - (*offset % 8))) & 1;
        }
        *offset += 1;
    }
    value
}

fn push_bits(bits: &mut Vec<bool>, value: u8, count: usize) {
    for shift in (0..count).rev() {
        bits.push(((value >> shift) & 1) == 1);
    }
}
