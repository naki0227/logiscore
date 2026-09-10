use crate::error::LogiscoreError;
use crate::protocol::midi_gen::{decode_from_midi, encode_to_midi};
use crate::protocol::Header;
use crate::transport::Transport;

/// Adapter that keeps the existing byte-to-note codec as the v2 Dense mode.
pub struct DenseMidiTransport {
    header: Header,
}

impl DenseMidiTransport {
    pub fn new(header: Header) -> Self {
        Self { header }
    }
}

impl Default for DenseMidiTransport {
    fn default() -> Self {
        Self {
            header: Header {
                scale_id: 0,
                root_key: 0,
                bytes_per_tick: 1,
            },
        }
    }
}

impl Transport for DenseMidiTransport {
    fn encode(&self, packet: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        encode_to_midi(packet, &self.header)
    }

    fn decode(&self, transport_data: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        let (_, packet) = decode_from_midi(transport_data)?;
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_transport_preserves_all_byte_values() {
        let original: Vec<u8> = (0..=u8::MAX).collect();
        let transport = DenseMidiTransport::default();
        let encoded = transport.encode(&original).unwrap();
        let decoded = transport.decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
