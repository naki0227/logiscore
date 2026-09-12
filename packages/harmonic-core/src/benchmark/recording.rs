use serde::{Deserialize, Serialize};

use super::channel::packet_bit_errors;
use crate::adaptive::{AcousticProfile, AcousticProfileId};
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, TextPayload};
use crate::v2::{build_profile_packet, decode_profile_packet};
use crate::v2_packet::decode_packet_bytes_for_fec_and_flags;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordingTelemetry {
    pub profile_id: Option<u8>,
    pub final_recovery_rate_percent: f64,
    pub raw_symbol_accuracy_percent: f64,
    pub corrected_errors: usize,
}

pub fn analyze_text_recording(
    samples: &[f32],
    sample_rate: u32,
    expected_text: &str,
) -> Result<RecordingTelemetry, LogiscoreError> {
    validate_recording(samples, sample_rate)?;
    let mut best = RecordingTelemetry {
        profile_id: None,
        final_recovery_rate_percent: 0.0,
        raw_symbol_accuracy_percent: 0.0,
        corrected_errors: 0,
    };
    for profile_id in profile_ids() {
        let profile = AcousticProfile::for_id(profile_id);
        let expected_packet =
            build_profile_packet(PayloadType::Text, expected_text.as_bytes(), profile)?;
        let Ok((raw_packet, codec_profile, _)) =
            decode_profile_packet(samples, sample_rate, profile_id)
        else {
            continue;
        };
        let (raw_symbol_accuracy_percent, bit_errors) =
            compare_packets(&expected_packet, &raw_packet);
        let final_recovery = decode_packet_bytes_for_fec_and_flags(
            &raw_packet,
            PayloadType::Text,
            codec_profile,
            profile.fec_profile,
            profile_id as u8,
        )
        .and_then(TextPayload::from_bytes)
        .is_ok_and(|payload| payload.as_str() == expected_text);
        let candidate = RecordingTelemetry {
            profile_id: Some(profile_id as u8),
            final_recovery_rate_percent: if final_recovery { 100.0 } else { 0.0 },
            raw_symbol_accuracy_percent,
            corrected_errors: if final_recovery { bit_errors } else { 0 },
        };
        if final_recovery {
            return Ok(candidate);
        }
        if candidate.raw_symbol_accuracy_percent > best.raw_symbol_accuracy_percent {
            best = candidate;
        }
    }
    Ok(best)
}

fn compare_packets(expected: &[u8], actual: &[u8]) -> (f64, usize) {
    if expected.len() != actual.len() {
        return (0.0, 0);
    }
    let bit_errors = packet_bit_errors(expected, actual);
    let total_bits = expected.len().saturating_mul(8);
    let accuracy = if total_bits == 0 {
        100.0
    } else {
        (total_bits - bit_errors) as f64 * 100.0 / total_bits as f64
    };
    (accuracy, bit_errors)
}

fn validate_recording(samples: &[f32], sample_rate: u32) -> Result<(), LogiscoreError> {
    if sample_rate == 0 {
        return Err(LogiscoreError::InvalidAudio(
            "recording sample rate must be positive".into(),
        ));
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(LogiscoreError::InvalidAudio(
            "recording contains non-finite samples".into(),
        ));
    }
    Ok(())
}

fn profile_ids() -> [AcousticProfileId; 7] {
    [
        AcousticProfileId::Quiet,
        AcousticProfileId::Balanced,
        AcousticProfileId::Conversation,
        AcousticProfileId::Noisy,
        AcousticProfileId::Online,
        AcousticProfileId::LongDistance,
        AcousticProfileId::FixedFallback,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::decode_wav;
    use crate::v2::encode_wav_with_profile;

    #[test]
    fn clean_recording_reports_complete_raw_and_final_recovery() {
        let wav = encode_wav_with_profile(PayloadType::Text, b"fixture", AcousticProfileId::Quiet)
            .unwrap();
        let audio = decode_wav(&wav).unwrap();
        let telemetry =
            analyze_text_recording(audio.samples(), audio.sample_rate(), "fixture").unwrap();
        assert_eq!(telemetry.profile_id, Some(AcousticProfileId::Quiet as u8));
        assert_eq!(telemetry.final_recovery_rate_percent, 100.0);
        assert_eq!(telemetry.raw_symbol_accuracy_percent, 100.0);
        assert_eq!(telemetry.corrected_errors, 0);
    }

    #[test]
    fn undetected_recording_reports_zero_without_claiming_a_profile() {
        let telemetry = analyze_text_recording(&[0.0; 800], 8_000, "fixture").unwrap();
        assert_eq!(telemetry.profile_id, None);
        assert_eq!(telemetry.final_recovery_rate_percent, 0.0);
        assert_eq!(telemetry.raw_symbol_accuracy_percent, 0.0);
    }

    #[test]
    fn rejects_invalid_recording_dimensions() {
        assert!(analyze_text_recording(&[0.0], 0, "fixture").is_err());
        assert!(analyze_text_recording(&[f32::NAN], 8_000, "fixture").is_err());
    }
}
