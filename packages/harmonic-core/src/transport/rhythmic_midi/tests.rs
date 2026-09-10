use super::*;
use crate::transport::musical_midi::MusicalMidiTransport;

#[test]
fn rhythmic_transport_roundtrips_every_byte_value() {
    let original = (0..=u8::MAX).collect::<Vec<_>>();
    let transport = RhythmicMidiTransport::default();
    let midi = transport.encode(&original).unwrap();
    assert_eq!(transport.decode(&midi).unwrap(), original);
}

#[test]
fn rhythmic_transport_is_deterministic_and_uses_multiple_durations() {
    let transport = RhythmicMidiTransport::default();
    let first = transport.encode(b"Logiscore rhythm").unwrap();
    let second = transport.encode(b"Logiscore rhythm").unwrap();
    assert_eq!(first, second);
    let (events, length) = parse_midi(&first).unwrap();
    let mut durations = events
        .iter()
        .map(|event| event.duration_ticks())
        .collect::<Vec<_>>();
    durations.sort_unstable();
    durations.dedup();
    assert_eq!(length, b"Logiscore rhythm".len());
    assert!(durations.len() > 1);
}

#[test]
fn rhythmic_transport_rejects_packet_over_limit() {
    assert!(RhythmicMidiTransport::default()
        .encode(&vec![0; MAX_PACKET_BYTES + 1])
        .is_err());
}

#[test]
fn rhythmic_transport_reduces_midi_size_against_profile_one() {
    let packet = (0..=u8::MAX).collect::<Vec<_>>();
    let baseline = MusicalMidiTransport::default().encode(&packet).unwrap();
    let rhythmic = RhythmicMidiTransport::default().encode(&packet).unwrap();
    assert!(rhythmic.len() < baseline.len());
}
