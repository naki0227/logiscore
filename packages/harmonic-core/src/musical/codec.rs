use super::{CandidateSet, ChordPlanner, MusicalError, TonalContext, Voicing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicalSymbolCodec {
    planner: ChordPlanner,
}

impl MusicalSymbolCodec {
    pub const fn new(context: TonalContext) -> Self {
        Self {
            planner: ChordPlanner::new(context),
        }
    }

    pub fn encode(&self, symbols: &[u8]) -> Result<Vec<Voicing>, MusicalError> {
        let mut previous = None;
        symbols
            .iter()
            .enumerate()
            .map(|(event_index, &symbol)| {
                let candidates =
                    CandidateSet::generate(self.planner.chord_at(event_index), previous)?;
                let voicing = candidates.encode_symbol(symbol)?;
                previous = Some(voicing);
                Ok(voicing)
            })
            .collect()
    }

    pub fn decode(&self, voicings: &[Voicing]) -> Result<Vec<u8>, MusicalError> {
        let mut previous = None;
        voicings
            .iter()
            .enumerate()
            .map(|(event_index, &voicing)| {
                let candidates =
                    CandidateSet::generate(self.planner.chord_at(event_index), previous)?;
                let symbol = candidates.decode_symbol(voicing)?;
                previous = Some(voicing);
                Ok(symbol)
            })
            .collect()
    }
}
