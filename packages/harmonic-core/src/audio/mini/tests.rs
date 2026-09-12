use super::*;
use crate::audio::FixedPcmCodec;

#[test]
fn mini_codec_roundtrips_multiple_frames_after_arbitrary_start() {
    let profile = PcmProfile::with_timing_percent(200).unwrap();
    let codec = MiniFixedPcmCodec::new(profile);
    let first = codec.encode(b"first").unwrap();
    let second = codec.encode(b"second").unwrap();
    let mut recording = first[first.len() / 2..].to_vec();
    recording.extend(second);
    let observations = codec
        .decode_all_observations_at_sample_rate(&recording, 8_000)
        .unwrap();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].bytes, b"second");
}

#[test]
fn mini_framing_is_shorter_than_legacy_fixed_framing() {
    let profile = PcmProfile::with_timing_percent(200).unwrap();
    let mini = MiniFixedPcmCodec::new(profile).encode(&[]).unwrap();
    let legacy = FixedPcmCodec::new(profile).encode(&[]).unwrap();
    assert!(mini.len() * 4 < legacy.len());
}

#[test]
fn mini_codec_rejects_packets_outside_its_length_field() {
    let profile = PcmProfile::default();
    let packet = vec![0; MAX_MINI_PACKET_BYTES + 1];
    assert!(MiniFixedPcmCodec::new(profile).encode(&packet).is_err());
}
