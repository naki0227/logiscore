mod candidates;
mod codec;
mod harmony;
mod rhythm;
mod tonal;

pub use candidates::{CandidateSet, Voice, Voicing, VOICES};
pub use codec::MusicalSymbolCodec;
pub use harmony::{Chord, ChordPlanner};
pub use rhythm::{RhythmicEvent, RhythmicMusicalCodec};
pub use tonal::{Mode, TonalContext};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MusicalError {
    #[error("root pitch class must be between 0 and 11, got {0}")]
    InvalidRoot(u8),
    #[error("symbol must be between 0 and 15, got {0}")]
    InvalidSymbol(u8),
    #[error("could not generate the requested number of valid musical candidates")]
    InsufficientCandidates,
    #[error("candidate count must be a power of two between 2 and 32, got {0}")]
    InvalidCandidateCount(usize),
    #[error("observed voicing is not in the current candidate set")]
    UnknownVoicing,
}

#[cfg(test)]
mod tests;
