use super::*;

struct IdentityTransport;

impl Transport for IdentityTransport {
    fn encode(&self, packet: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        Ok(packet.to_vec())
    }

    fn decode(&self, transport_data: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        Ok(transport_data.to_vec())
    }
}

#[test]
fn decoder_rejects_trailing_packet_data() {
    let mut packet = encode_packet(PayloadType::Text, b"test", &IdentityTransport).unwrap();
    packet.push(0);
    assert!(decode_packet(&packet, PayloadType::Text, &IdentityTransport).is_err());
}

#[test]
fn decoder_rejects_unimplemented_profiles() {
    let mut packet = encode_packet(PayloadType::Text, b"test", &IdentityTransport).unwrap();
    packet[2] |= 1 << 3;
    assert!(decode_packet(&packet, PayloadType::Text, &IdentityTransport).is_err());
}

#[test]
fn reliable_packet_corrects_a_post_vote_bit_error() {
    let payload = b"reliable packet";
    let mut packet = build_packet_for_fec(
        PayloadType::Text,
        payload,
        RHYTHMIC_CODEC_PROFILE,
        RELIABLE_FEC_PROFILE,
    )
    .unwrap();
    let copy_length = (packet.len() - HEADER_LENGTH) / 3;
    packet[HEADER_LENGTH] ^= 0x80;
    packet[HEADER_LENGTH + copy_length] ^= 0x80;
    assert_eq!(
        decode_packet_bytes_for_fec(
            &packet,
            PayloadType::Text,
            RHYTHMIC_CODEC_PROFILE,
            RELIABLE_FEC_PROFILE,
        )
        .unwrap(),
        payload
    );
}

#[test]
fn reliable_and_unprotected_profiles_are_not_interchangeable() {
    let packet = build_packet_for_fec(
        PayloadType::Text,
        b"profile",
        RHYTHMIC_CODEC_PROFILE,
        RELIABLE_FEC_PROFILE,
    )
    .unwrap();
    assert!(decode_packet_bytes(&packet, PayloadType::Text, RHYTHMIC_CODEC_PROFILE).is_err());
}
