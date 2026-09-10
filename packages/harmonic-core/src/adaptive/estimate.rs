use super::AcousticProfile;
use crate::musical::RhythmicMusicalCodec;

const BASE_FRAMING_MS: u64 = 1_880;
const AVERAGE_EVENT_MS: u64 = 190;

pub fn estimate_duration_ms(profile: AcousticProfile, packet_bytes: usize) -> u64 {
    let events = RhythmicMusicalCodec::event_count_for_bytes(packet_bytes) as u64;
    (BASE_FRAMING_MS + events.saturating_mul(AVERAGE_EVENT_MS))
        .saturating_mul(u64::from(profile.timing_percent))
        / 100
}

pub fn estimate_payload_duration_ms(profile: AcousticProfile, payload_bytes: usize) -> u64 {
    let compressed_upper_estimate = payload_bytes.saturating_add(24);
    let body_bytes = if profile.fec_profile == 0 {
        compressed_upper_estimate
    } else {
        (compressed_upper_estimate
            .saturating_add(4)
            .saturating_mul(13)
            .saturating_add(7)
            / 8)
        .saturating_mul(usize::from(profile.repetition))
    };
    estimate_duration_ms(profile, body_bytes.saturating_add(7))
}
