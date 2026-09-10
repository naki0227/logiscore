use crate::error::LogiscoreError;

pub mod dense_midi;
pub mod musical_midi;
pub mod rhythmic_midi;

/// Transport boundary shared by Dense MIDI and future Musical/Acoustic codecs.
pub trait Transport {
    fn encode(&self, packet: &[u8]) -> Result<Vec<u8>, LogiscoreError>;
    fn decode(&self, transport_data: &[u8]) -> Result<Vec<u8>, LogiscoreError>;
}
