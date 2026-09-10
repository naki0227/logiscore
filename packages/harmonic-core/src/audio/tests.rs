use super::*;

#[test]
fn clean_pcm_roundtrips_packet() {
    let codec = PcmCodec::default();
    let packet = b"Logiscore PCM";
    let samples = codec.encode(packet).unwrap();
    assert_eq!(codec.decode(&samples).unwrap(), packet);
}

#[test]
fn timing_recovery_tolerates_leading_offset() {
    let codec = PcmCodec::default();
    let packet = [0x00, 0xff, 0x5a, 0xa5];
    let encoded = codec.encode(&packet).unwrap();
    let mut shifted = vec![0.0; 337];
    shifted.extend(encoded);
    assert_eq!(codec.decode(&shifted).unwrap(), packet);
}

#[test]
fn decoder_tolerates_low_deterministic_noise() {
    let codec = PcmCodec::default();
    let packet = b"noise";
    let mut samples = codec.encode(packet).unwrap();
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample += ((index as f32 * 0.37).sin()) * 0.002;
    }
    assert_eq!(codec.decode(&samples).unwrap(), packet);
}

#[test]
fn decoder_rejects_missing_preamble_and_non_finite_input() {
    let codec = PcmCodec::default();
    assert!(codec.decode(&vec![0.0; 8_000]).is_err());
    assert!(codec.decode(&[f32::NAN]).is_err());
}

#[test]
fn encoder_rejects_packet_over_pcm_limit() {
    assert!(PcmCodec::default()
        .encode(&vec![0; MAX_PCM_PACKET_BYTES + 1])
        .is_err());
}
