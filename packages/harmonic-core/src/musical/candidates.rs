use super::{Chord, MusicalError};

pub const VOICES: [Voice; 4] = [
    Voice::new("Bass", 36, 52),
    Voice::new("Harmony 2", 48, 64),
    Voice::new("Harmony 1", 55, 72),
    Voice::new("Melody", 60, 84),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voice {
    name: &'static str,
    min_note: u8,
    max_note: u8,
}

impl Voice {
    const fn new(name: &'static str, min_note: u8, max_note: u8) -> Self {
        Self {
            name,
            min_note,
            max_note,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn contains(self, note: u8) -> bool {
        note >= self.min_note && note <= self.max_note
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Voicing {
    notes: [u8; 4],
}

impl Voicing {
    pub(crate) const fn from_notes(notes: [u8; 4]) -> Self {
        Self { notes }
    }

    pub const fn notes(self) -> [u8; 4] {
        self.notes
    }

    pub fn is_valid_for(self, chord: Chord) -> bool {
        self.notes
            .iter()
            .zip(VOICES)
            .all(|(&note, voice)| voice.contains(note) && chord.contains(note))
            && self.notes.windows(2).all(|pair| pair[0] < pair[1])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSet {
    candidates: Vec<Voicing>,
}

impl CandidateSet {
    pub fn generate(chord: Chord, previous: Option<Voicing>) -> Result<Self, MusicalError> {
        Self::generate_with_count(chord, previous, 16)
    }

    pub fn generate_with_count(
        chord: Chord,
        previous: Option<Voicing>,
        candidate_count: usize,
    ) -> Result<Self, MusicalError> {
        if !(2..=32).contains(&candidate_count) || !candidate_count.is_power_of_two() {
            return Err(MusicalError::InvalidCandidateCount(candidate_count));
        }
        let notes = VOICES.map(|voice| notes_for_voice(voice, chord));
        let mut scored = Vec::new();
        for &bass in &notes[0] {
            for &harmony_2 in &notes[1] {
                for &harmony_1 in &notes[2] {
                    for &melody in &notes[3] {
                        let voicing = Voicing {
                            notes: [bass, harmony_2, harmony_1, melody],
                        };
                        if voicing.is_valid_for(chord) {
                            scored.push((score(voicing, previous), voicing));
                        }
                    }
                }
            }
        }
        scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
        scored.dedup_by_key(|(_, voicing)| *voicing);
        let candidates = scored
            .into_iter()
            .take(candidate_count)
            .map(|(_, voicing)| voicing)
            .collect::<Vec<_>>();
        if candidates.len() != candidate_count {
            return Err(MusicalError::InsufficientCandidates);
        }
        Ok(Self { candidates })
    }

    pub fn candidates(&self) -> &[Voicing] {
        &self.candidates
    }

    pub fn encode_symbol(&self, symbol: u8) -> Result<Voicing, MusicalError> {
        self.candidates
            .get(usize::from(symbol))
            .copied()
            .ok_or(MusicalError::InvalidSymbol(symbol))
    }

    pub fn decode_symbol(&self, observed: Voicing) -> Result<u8, MusicalError> {
        self.candidates
            .iter()
            .position(|candidate| *candidate == observed)
            .map(|index| index as u8)
            .ok_or(MusicalError::UnknownVoicing)
    }
}

fn notes_for_voice(voice: Voice, chord: Chord) -> Vec<u8> {
    (voice.min_note..=voice.max_note)
        .filter(|note| chord.contains(*note))
        .collect()
}

fn score(voicing: Voicing, previous: Option<Voicing>) -> i32 {
    let center_score = voicing
        .notes
        .iter()
        .zip(VOICES)
        .map(|(&note, voice)| {
            let center = (i32::from(voice.min_note) + i32::from(voice.max_note)) / 2;
            12 - (i32::from(note) - center).abs()
        })
        .sum::<i32>();
    let movement_score = previous.map_or(0, |prior| {
        voicing
            .notes
            .iter()
            .zip(prior.notes)
            .map(|(&note, old)| motion_score(note.abs_diff(old)))
            .sum()
    });
    center_score + movement_score + consonance_score(voicing) + spacing_score(voicing)
}

fn motion_score(distance: u8) -> i32 {
    match distance {
        0 => 10,
        1 | 2 => 8,
        3 | 4 => 5,
        5..=7 => 1,
        _ => -i32::from(distance),
    }
}

fn consonance_score(voicing: Voicing) -> i32 {
    let mut total = 0;
    for lower in 0..voicing.notes.len() {
        for upper in (lower + 1)..voicing.notes.len() {
            total += match (voicing.notes[upper] - voicing.notes[lower]) % 12 {
                0 | 3 | 4 | 5 | 7 | 8 | 9 => 3,
                1 | 2 | 6 | 10 | 11 => -4,
                _ => 0,
            };
        }
    }
    total
}

fn spacing_score(voicing: Voicing) -> i32 {
    let bass_gap = voicing.notes[1] - voicing.notes[0];
    let upper_gaps = [
        voicing.notes[2] - voicing.notes[1],
        voicing.notes[3] - voicing.notes[2],
    ];
    let bass_score = if (5..=19).contains(&bass_gap) { 3 } else { -5 };
    bass_score
        + upper_gaps
            .into_iter()
            .map(|gap| if gap <= 12 { 2 } else { -3 })
            .sum::<i32>()
}
