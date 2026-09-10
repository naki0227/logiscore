use crate::error::LogiscoreError;
use crate::payload::PayloadType;

pub const HEADER_LENGTH: usize = 7;
pub const PROTOCOL_VERSION: u8 = 2;
const MAGIC: u16 = 0x4C5;
const MAX_PAYLOAD_LENGTH: usize = 0x00FF_FFFF;

/// Compact transport-neutral header defined by the v2 protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryHeader {
    pub payload_type: PayloadType,
    pub codec_profile: u8,
    pub fec_profile: u8,
    pub flags: u8,
    pub payload_length: usize,
}

impl BinaryHeader {
    pub fn new(
        payload_type: PayloadType,
        codec_profile: u8,
        fec_profile: u8,
        flags: u8,
        payload_length: usize,
    ) -> Result<Self, LogiscoreError> {
        if codec_profile > 0b111 {
            return Err(LogiscoreError::InvalidPacket(format!(
                "codec profile exceeds 3 bits: {codec_profile}"
            )));
        }
        if fec_profile > 0b111 {
            return Err(LogiscoreError::InvalidPacket(format!(
                "FEC profile exceeds 3 bits: {fec_profile}"
            )));
        }
        if payload_length > MAX_PAYLOAD_LENGTH {
            return Err(LogiscoreError::InvalidPacket(format!(
                "payload exceeds 24-bit length: {payload_length}"
            )));
        }

        Ok(Self {
            payload_type,
            codec_profile,
            fec_profile,
            flags,
            payload_length,
        })
    }

    pub fn encode(self) -> [u8; HEADER_LENGTH] {
        [
            (MAGIC >> 4) as u8,
            ((MAGIC as u8 & 0x0f) << 4) | PROTOCOL_VERSION,
            ((self.payload_type as u8) << 6) | (self.codec_profile << 3) | self.fec_profile,
            self.flags,
            ((self.payload_length >> 16) & 0xff) as u8,
            ((self.payload_length >> 8) & 0xff) as u8,
            (self.payload_length & 0xff) as u8,
        ]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, LogiscoreError> {
        if bytes.len() < HEADER_LENGTH {
            return Err(LogiscoreError::InvalidPacket(format!(
                "header truncated: expected {HEADER_LENGTH} bytes, got {}",
                bytes.len()
            )));
        }

        let magic = (u16::from(bytes[0]) << 4) | u16::from(bytes[1] >> 4);
        if magic != MAGIC {
            return Err(LogiscoreError::InvalidPacket("magic mismatch".into()));
        }

        let version = bytes[1] & 0x0f;
        if version != PROTOCOL_VERSION {
            return Err(LogiscoreError::UnsupportedVersion(format!("v{version}")));
        }

        let payload_type = PayloadType::try_from(bytes[2] >> 6)?;
        let codec_profile = (bytes[2] >> 3) & 0b111;
        let fec_profile = bytes[2] & 0b111;
        let payload_length =
            (usize::from(bytes[4]) << 16) | (usize::from(bytes[5]) << 8) | usize::from(bytes[6]);

        Self::new(
            payload_type,
            codec_profile,
            fec_profile,
            bytes[3],
            payload_length,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_header_roundtrips_boundaries() {
        for payload_length in [0, 1, MAX_PAYLOAD_LENGTH] {
            let header = BinaryHeader::new(PayloadType::Text, 7, 7, 0xff, payload_length).unwrap();
            assert_eq!(BinaryHeader::decode(&header.encode()).unwrap(), header);
        }
    }

    #[test]
    fn binary_header_rejects_truncated_data() {
        assert!(BinaryHeader::decode(&[0; HEADER_LENGTH - 1]).is_err());
    }

    #[test]
    fn binary_header_rejects_invalid_magic() {
        let mut encoded = BinaryHeader::new(PayloadType::Text, 0, 0, 0, 1)
            .unwrap()
            .encode();
        encoded[0] ^= 1;
        assert!(BinaryHeader::decode(&encoded).is_err());
    }

    #[test]
    fn binary_header_rejects_unsupported_version() {
        let mut encoded = BinaryHeader::new(PayloadType::Text, 0, 0, 0, 1)
            .unwrap()
            .encode();
        encoded[1] = (encoded[1] & 0xf0) | 3;
        assert!(matches!(
            BinaryHeader::decode(&encoded),
            Err(LogiscoreError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn binary_header_rejects_out_of_range_fields() {
        assert!(BinaryHeader::new(PayloadType::Text, 8, 0, 0, 0).is_err());
        assert!(BinaryHeader::new(PayloadType::Text, 0, 8, 0, 0).is_err());
        assert!(BinaryHeader::new(PayloadType::Text, 0, 0, 0, MAX_PAYLOAD_LENGTH + 1).is_err());
    }
}
