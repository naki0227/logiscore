use std::time::Instant;

use serde::Serialize;

use crate::adaptive::{AcousticProfile, AcousticProfileId, Environment};
use crate::audio::decode_wav;
use crate::error::LogiscoreError;
use crate::payload::PayloadType;
use crate::v2::{decode_text_wav_adaptive, encode_text_wav_secure, encode_wav_with_profile};
use crate::v2_packet::decode_packet_bytes_for_fec_and_flags;

const SAMPLE_RATE: u32 = 8_000;
const MAX_ITERATIONS: usize = 100;
const REPORT_SCHEMA_VERSION: u8 = 2;

mod channel;
mod midi;

#[derive(Debug, Serialize)]
pub struct BenchmarkReport {
    pub schema_version: u8,
    pub fixture_bytes: usize,
    pub iterations: usize,
    pub rows: Vec<BenchmarkRow>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkRow {
    pub name: String,
    pub transport: &'static str,
    pub output_bytes: usize,
    pub encode_median_micros: u128,
    pub decode_median_micros: u128,
    pub playback_ms: Option<u64>,
    pub payload_bitrate_bps: Option<f64>,
    pub clean_roundtrip: bool,
    pub final_recovery_rate_percent: f64,
    pub raw_symbol_accuracy_percent: f64,
    pub corrected_errors: usize,
    pub music_weight: Option<u8>,
    pub max_polyphony: Option<u8>,
    pub fec_profile: Option<u8>,
    pub channel_successes: Option<usize>,
    pub channel_cases: Option<usize>,
}

pub fn run_text_benchmark(
    text: &str,
    password: &str,
    iterations: usize,
) -> Result<BenchmarkReport, LogiscoreError> {
    if !(1..=MAX_ITERATIONS).contains(&iterations) {
        return Err(LogiscoreError::InvalidProfile(format!(
            "benchmark iterations must be between 1 and {MAX_ITERATIONS}"
        )));
    }
    let mut rows = vec![
        midi::dense(text, iterations)?,
        midi::rhythmic(text, iterations)?,
    ];
    for profile_id in profile_ids() {
        rows.push(benchmark_profile(text, iterations, profile_id)?);
    }
    rows.push(benchmark_secure(text, password, iterations)?);
    Ok(BenchmarkReport {
        schema_version: REPORT_SCHEMA_VERSION,
        fixture_bytes: text.len(),
        iterations,
        rows,
    })
}

fn benchmark_profile(
    text: &str,
    iterations: usize,
    profile_id: AcousticProfileId,
) -> Result<BenchmarkRow, LogiscoreError> {
    let profile = AcousticProfile::for_id(profile_id);
    let (wav, encode_median_micros) = measure(iterations, || {
        encode_wav_with_profile(PayloadType::Text, text.as_bytes(), profile_id)
    })?;
    let (decoded, decode_median_micros) = measure(iterations, || decode_text_wav_adaptive(&wav))?;
    let playback_ms = wav_duration_ms(&wav)?;
    let clean_audio = decode_wav(&wav)?;
    let (expected_packet, codec_profile, _) = crate::v2::decode_profile_packet(
        clean_audio.samples(),
        clean_audio.sample_rate(),
        profile_id,
    )?;
    let channel_metrics = channel::measure(&wav, &expected_packet, |samples| {
        let Ok((packet, _, _)) = crate::v2::decode_profile_packet(samples, SAMPLE_RATE, profile_id)
        else {
            return channel::ChannelObservation {
                raw_packet: None,
                final_recovery: false,
            };
        };
        let final_recovery = decode_packet_bytes_for_fec_and_flags(
            &packet,
            PayloadType::Text,
            codec_profile,
            profile.fec_profile,
            profile_id as u8,
        )
        .and_then(crate::payload::TextPayload::from_bytes)
        .is_ok_and(|payload| payload.as_str() == text);
        channel::ChannelObservation {
            raw_packet: Some(packet),
            final_recovery,
        }
    })?;
    Ok(audio_row(AudioMeasurement {
        name: format!("{:?}", profile_id),
        output_bytes: wav.len(),
        encode_median_micros,
        decode_median_micros,
        playback_ms,
        payload_bytes: text.len(),
        clean_roundtrip: decoded.payload == text,
        profile,
        channel_metrics,
    }))
}

fn benchmark_secure(
    text: &str,
    password: &str,
    iterations: usize,
) -> Result<BenchmarkRow, LogiscoreError> {
    let (encoded, encode_median_micros) = measure(iterations, || {
        encode_text_wav_secure(text, password, Environment::Auto, 50)
    })?;
    let (decoded, decode_median_micros) = measure(iterations, || {
        crate::v2::decode_wav_secure_auto(&encoded.bytes, password)
    })?;
    let playback_ms = wav_duration_ms(&encoded.bytes)?;
    let clean_audio = decode_wav(&encoded.bytes)?;
    let profile_id = encoded.selection.primary.id;
    let (expected_packet, codec_profile, profile) = crate::v2::decode_profile_packet(
        clean_audio.samples(),
        clean_audio.sample_rate(),
        profile_id,
    )?;
    let channel_metrics = channel::measure(&encoded.bytes, &expected_packet, |samples| {
        let Ok((packet, _, _)) = crate::v2::decode_profile_packet(samples, SAMPLE_RATE, profile_id)
        else {
            return channel::ChannelObservation {
                raw_packet: None,
                final_recovery: false,
            };
        };
        let final_recovery = crate::v2_secure_packet::decode_secure_packet_for_fec_and_flags(
            &packet,
            password,
            PayloadType::Text,
            codec_profile,
            profile.fec_profile,
            profile_id as u8,
        )
        .and_then(crate::payload::TextPayload::from_bytes)
        .is_ok_and(|payload| payload.as_str() == text);
        channel::ChannelObservation {
            raw_packet: Some(packet),
            final_recovery,
        }
    })?;
    Ok(audio_row(AudioMeasurement {
        name: format!("Secure {:?}", encoded.selection.primary.id),
        output_bytes: encoded.bytes.len(),
        encode_median_micros,
        decode_median_micros,
        playback_ms,
        payload_bytes: text.len(),
        clean_roundtrip: matches!(decoded.payload, crate::v2::SecurePayload::Text(value) if value == text),
        profile: encoded.selection.primary,
        channel_metrics,
    }))
}

fn measure<T>(
    iterations: usize,
    mut operation: impl FnMut() -> Result<T, LogiscoreError>,
) -> Result<(T, u128), LogiscoreError> {
    let mut timings = Vec::with_capacity(iterations);
    let mut result = operation()?;
    for _ in 0..iterations {
        let start = Instant::now();
        result = operation()?;
        timings.push(start.elapsed().as_micros());
    }
    timings.sort_unstable();
    Ok((result, timings[timings.len() / 2]))
}

fn wav_duration_ms(wav: &[u8]) -> Result<u64, LogiscoreError> {
    let audio = decode_wav(wav)?;
    Ok((audio.samples().len() as u64 * 1_000) / u64::from(audio.sample_rate()))
}

struct AudioMeasurement {
    name: String,
    output_bytes: usize,
    encode_median_micros: u128,
    decode_median_micros: u128,
    playback_ms: u64,
    payload_bytes: usize,
    clean_roundtrip: bool,
    profile: AcousticProfile,
    channel_metrics: channel::ChannelMetrics,
}

fn audio_row(measurement: AudioMeasurement) -> BenchmarkRow {
    BenchmarkRow {
        name: measurement.name,
        transport: "WAV",
        output_bytes: measurement.output_bytes,
        encode_median_micros: measurement.encode_median_micros,
        decode_median_micros: measurement.decode_median_micros,
        playback_ms: Some(measurement.playback_ms),
        payload_bitrate_bps: Some(
            measurement.payload_bytes as f64 * 8_000.0 / measurement.playback_ms as f64,
        ),
        clean_roundtrip: measurement.clean_roundtrip,
        final_recovery_rate_percent: measurement.channel_metrics.final_recoveries as f64 * 100.0
            / measurement.channel_metrics.cases as f64,
        raw_symbol_accuracy_percent: measurement.channel_metrics.raw_symbol_accuracy_percent,
        corrected_errors: measurement.channel_metrics.corrected_errors,
        music_weight: Some(measurement.profile.music_weight),
        max_polyphony: Some(measurement.profile.max_polyphony),
        fec_profile: Some(measurement.profile.fec_profile),
        channel_successes: Some(measurement.channel_metrics.final_recoveries),
        channel_cases: Some(measurement.channel_metrics.cases),
    }
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

    #[test]
    fn benchmark_reports_every_transport_with_clean_roundtrips() {
        let report = run_text_benchmark("benchmark fixture", "password", 1).unwrap();
        assert_eq!(report.schema_version, 2);
        assert_eq!(report.rows.len(), 10);
        assert!(report.rows.iter().all(|row| row.clean_roundtrip));
        assert!(report
            .rows
            .iter()
            .filter(|row| row.transport == "WAV")
            .all(|row| {
                row.playback_ms.unwrap_or_default() > 0
                    && row.payload_bitrate_bps.unwrap_or_default() > 0.0
                    && row.channel_cases == Some(5)
                    && (0.0..=100.0).contains(&row.final_recovery_rate_percent)
                    && (0.0..=100.0).contains(&row.raw_symbol_accuracy_percent)
            }));
    }

    #[test]
    fn benchmark_rejects_unbounded_iterations() {
        assert!(run_text_benchmark("fixture", "password", 0).is_err());
        assert!(run_text_benchmark("fixture", "password", 101).is_err());
    }
}
