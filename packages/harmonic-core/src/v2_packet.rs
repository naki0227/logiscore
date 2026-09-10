use crate::compressor;
use crate::error::LogiscoreError;
use crate::error_correction;
use crate::payload::PayloadType;
use crate::protocol::v2::{BinaryHeader, HEADER_LENGTH};
use crate::transport::Transport;

pub(crate) const DENSE_CODEC_PROFILE: u8 = 0;
pub(crate) const MUSICAL_CODEC_PROFILE: u8 = 1;
pub(crate) const RHYTHMIC_CODEC_PROFILE: u8 = 2;
pub(crate) const FIXED_AUDIO_CODEC_PROFILE: u8 = 3;
const NO_FEC_PROFILE: u8 = 0;
pub(crate) const RELIABLE_FEC_PROFILE: u8 = 1;
const NO_FLAGS: u8 = 0;
pub(crate) const MAX_CANONICAL_PAYLOAD_LENGTH: usize = 36 * 1024 * 1024;
const MAX_PACKET_BODY_LENGTH: usize = 0x00ff_ffff;

pub(crate) fn encode_packet<T: Transport>(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    encode_packet_for_profile(
        payload_type,
        canonical_payload,
        DENSE_CODEC_PROFILE,
        transport,
    )
}

pub(crate) fn encode_packet_for_profile<T: Transport>(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    codec_profile: u8,
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    transport.encode(&build_packet(
        payload_type,
        canonical_payload,
        codec_profile,
    )?)
}

pub(crate) fn build_packet(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    codec_profile: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    build_packet_for_fec(
        payload_type,
        canonical_payload,
        codec_profile,
        NO_FEC_PROFILE,
    )
}

pub(crate) fn build_packet_for_fec(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    codec_profile: u8,
    fec_profile: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    build_packet_for_fec_and_flags(
        payload_type,
        canonical_payload,
        codec_profile,
        fec_profile,
        NO_FLAGS,
    )
}

pub(crate) fn build_packet_for_fec_and_flags(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    codec_profile: u8,
    fec_profile: u8,
    flags: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    if canonical_payload.len() > MAX_CANONICAL_PAYLOAD_LENGTH {
        return Err(LogiscoreError::InvalidPacket(format!(
            "canonical payload exceeds {MAX_CANONICAL_PAYLOAD_LENGTH} bytes"
        )));
    }
    build_packet_from_body(
        payload_type,
        &compressor::compress(canonical_payload)?,
        codec_profile,
        fec_profile,
        flags,
    )
}

pub(crate) fn build_packet_from_body(
    payload_type: PayloadType,
    unprotected_body: &[u8],
    codec_profile: u8,
    fec_profile: u8,
    flags: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    let body = match fec_profile {
        NO_FEC_PROFILE => unprotected_body.to_vec(),
        1..=3 => {
            let protected_length = error_correction::protected_length_for_profile(
                unprotected_body.len(),
                fec_profile,
            )?;
            if protected_length > MAX_PACKET_BODY_LENGTH {
                return Err(LogiscoreError::InvalidFec(
                    "protected payload exceeds the v2 packet length field".into(),
                ));
            }
            error_correction::protect_for_profile(unprotected_body, fec_profile)?
        }
        _ => {
            return Err(LogiscoreError::InvalidFec(format!(
                "unsupported FEC profile: {fec_profile}"
            )))
        }
    };
    let header = BinaryHeader::new(payload_type, codec_profile, fec_profile, flags, body.len())?;
    let mut packet = Vec::with_capacity(HEADER_LENGTH + body.len());
    packet.extend_from_slice(&header.encode());
    packet.extend_from_slice(&body);
    Ok(packet)
}

pub(crate) fn decode_packet<T: Transport>(
    transport_data: &[u8],
    expected_payload_type: PayloadType,
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    decode_packet_for_profile(
        transport_data,
        expected_payload_type,
        DENSE_CODEC_PROFILE,
        transport,
    )
}

pub(crate) fn decode_packet_for_profile<T: Transport>(
    transport_data: &[u8],
    expected_payload_type: PayloadType,
    expected_codec_profile: u8,
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    let packet = transport.decode(transport_data)?;
    decode_packet_bytes(&packet, expected_payload_type, expected_codec_profile)
}

pub(crate) fn decode_packet_bytes(
    packet: &[u8],
    expected_payload_type: PayloadType,
    expected_codec_profile: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    decode_packet_bytes_for_fec(
        packet,
        expected_payload_type,
        expected_codec_profile,
        NO_FEC_PROFILE,
    )
}

pub(crate) fn decode_packet_bytes_for_fec(
    packet: &[u8],
    expected_payload_type: PayloadType,
    expected_codec_profile: u8,
    expected_fec_profile: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    decode_packet_bytes_for_fec_and_flags(
        packet,
        expected_payload_type,
        expected_codec_profile,
        expected_fec_profile,
        NO_FLAGS,
    )
}

pub(crate) fn decode_packet_bytes_for_fec_and_flags(
    packet: &[u8],
    expected_payload_type: PayloadType,
    expected_codec_profile: u8,
    expected_fec_profile: u8,
    expected_flags: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    let compressed = decode_packet_body_for_fec_and_flags(
        packet,
        expected_payload_type,
        expected_codec_profile,
        expected_fec_profile,
        expected_flags,
    )?;
    compressor::decompress_limited(&compressed, MAX_CANONICAL_PAYLOAD_LENGTH).map_err(Into::into)
}

pub(crate) fn decode_packet_body_for_fec_and_flags(
    packet: &[u8],
    expected_payload_type: PayloadType,
    expected_codec_profile: u8,
    expected_fec_profile: u8,
    expected_flags: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    let header = BinaryHeader::decode(packet)?;
    if header.payload_type != expected_payload_type {
        return Err(LogiscoreError::InvalidPacket(format!(
            "expected {expected_payload_type:?} payload, got {:?}",
            header.payload_type,
        )));
    }
    if header.codec_profile != expected_codec_profile
        || header.fec_profile != expected_fec_profile
        || header.flags != expected_flags
    {
        return Err(LogiscoreError::InvalidPacket(format!(
            "unsupported v2 profile: codec={}, fec={}, flags={:#04x}",
            header.codec_profile, header.fec_profile, header.flags
        )));
    }
    let expected_length = HEADER_LENGTH
        .checked_add(header.payload_length)
        .ok_or_else(|| LogiscoreError::InvalidPacket("packet length overflow".into()))?;
    if packet.len() != expected_length {
        return Err(LogiscoreError::InvalidPacket(format!(
            "packet length mismatch: expected {expected_length}, got {}",
            packet.len()
        )));
    }
    let body = match header.fec_profile {
        NO_FEC_PROFILE => packet[HEADER_LENGTH..].to_vec(),
        1..=3 => {
            error_correction::recover_for_profile(&packet[HEADER_LENGTH..], header.fec_profile)?
                .into_data()
        }
        _ => {
            return Err(LogiscoreError::InvalidFec(format!(
                "unsupported FEC profile: {}",
                header.fec_profile
            )))
        }
    };
    Ok(body)
}

#[cfg(test)]
mod tests;
