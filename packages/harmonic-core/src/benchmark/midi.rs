use super::{measure, BenchmarkRow};
use crate::error::LogiscoreError;
use crate::v2::{decode_text_dense, decode_text_rhythmic, encode_text_dense, encode_text_rhythmic};

pub(super) fn dense(text: &str, iterations: usize) -> Result<BenchmarkRow, LogiscoreError> {
    let (midi, encode_median_micros) = measure(iterations, || encode_text_dense(text))?;
    let (decoded, decode_median_micros) = measure(iterations, || decode_text_dense(&midi))?;
    Ok(row(
        "Dense MIDI",
        midi.len(),
        encode_median_micros,
        decode_median_micros,
        decoded == text,
    ))
}

pub(super) fn rhythmic(text: &str, iterations: usize) -> Result<BenchmarkRow, LogiscoreError> {
    let (midi, encode_median_micros) = measure(iterations, || encode_text_rhythmic(text))?;
    let (decoded, decode_median_micros) = measure(iterations, || decode_text_rhythmic(&midi))?;
    Ok(row(
        "Rhythmic MIDI",
        midi.len(),
        encode_median_micros,
        decode_median_micros,
        decoded == text,
    ))
}

fn row(
    name: &str,
    output_bytes: usize,
    encode_median_micros: u128,
    decode_median_micros: u128,
    clean_roundtrip: bool,
) -> BenchmarkRow {
    let digital_accuracy = if clean_roundtrip { 100.0 } else { 0.0 };
    BenchmarkRow {
        name: name.to_owned(),
        transport: "MIDI",
        output_bytes,
        encode_median_micros,
        decode_median_micros,
        playback_ms: None,
        payload_bitrate_bps: None,
        clean_roundtrip,
        final_recovery_rate_percent: digital_accuracy,
        raw_symbol_accuracy_percent: digital_accuracy,
        corrected_errors: 0,
        music_weight: None,
        max_polyphony: None,
        fec_profile: None,
        channel_successes: None,
        channel_cases: None,
    }
}
