use crate::compressor;
use crate::error::LogiscoreError;
use crate::payload::PayloadType;
use crate::secure;
use crate::v2_packet::{
    build_packet_from_body, decode_packet_body_for_fec_and_flags, MAX_CANONICAL_PAYLOAD_LENGTH,
};

pub(crate) const SECURE_FLAG: u8 = 0x80;

pub(crate) fn build_secure_packet_for_fec_and_flags(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    password: &str,
    codec_profile: u8,
    fec_profile: u8,
    profile_flags: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    validate_flags(profile_flags)?;
    if canonical_payload.len() > MAX_CANONICAL_PAYLOAD_LENGTH {
        return Err(LogiscoreError::InvalidPacket(format!(
            "canonical payload exceeds {MAX_CANONICAL_PAYLOAD_LENGTH} bytes"
        )));
    }
    let compressed = compressor::compress(canonical_payload)?;
    let envelope = secure::seal(&compressed, password)?;
    build_packet_from_body(
        payload_type,
        envelope.as_bytes(),
        codec_profile,
        fec_profile,
        profile_flags | SECURE_FLAG,
    )
}

pub(crate) fn decode_secure_packet_for_fec_and_flags(
    packet: &[u8],
    password: &str,
    expected_payload_type: PayloadType,
    expected_codec_profile: u8,
    expected_fec_profile: u8,
    profile_flags: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    validate_flags(profile_flags)?;
    let envelope = decode_packet_body_for_fec_and_flags(
        packet,
        expected_payload_type,
        expected_codec_profile,
        expected_fec_profile,
        profile_flags | SECURE_FLAG,
    )?;
    let compressed = secure::open(&envelope, password)?;
    compressor::decompress_limited(&compressed, MAX_CANONICAL_PAYLOAD_LENGTH).map_err(Into::into)
}

fn validate_flags(profile_flags: u8) -> Result<(), LogiscoreError> {
    if profile_flags & SECURE_FLAG != 0 {
        return Err(LogiscoreError::InvalidPacket(
            "secure flag overlaps profile flags".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2_packet::{RELIABLE_FEC_PROFILE, RHYTHMIC_CODEC_PROFILE};

    #[test]
    fn secure_packet_roundtrips_after_fec_recovery() {
        let packet = build_secure_packet_for_fec_and_flags(
            PayloadType::Text,
            "暗号化 🎼".as_bytes(),
            "password",
            RHYTHMIC_CODEC_PROFILE,
            RELIABLE_FEC_PROFILE,
            2,
        )
        .unwrap();
        assert_eq!(
            decode_secure_packet_for_fec_and_flags(
                &packet,
                "password",
                PayloadType::Text,
                RHYTHMIC_CODEC_PROFILE,
                RELIABLE_FEC_PROFILE,
                2,
            )
            .unwrap(),
            "暗号化 🎼".as_bytes()
        );
    }

    #[test]
    fn normal_decoder_cannot_open_secure_packet() {
        let packet = build_secure_packet_for_fec_and_flags(
            PayloadType::Text,
            b"secret",
            "password",
            RHYTHMIC_CODEC_PROFILE,
            RELIABLE_FEC_PROFILE,
            2,
        )
        .unwrap();
        assert!(crate::v2_packet::decode_packet_bytes_for_fec_and_flags(
            &packet,
            PayloadType::Text,
            RHYTHMIC_CODEC_PROFILE,
            RELIABLE_FEC_PROFILE,
            2,
        )
        .is_err());
    }
}
