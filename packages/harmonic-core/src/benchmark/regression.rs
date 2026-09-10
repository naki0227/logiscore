use serde::Serialize;

use super::{BenchmarkReport, BenchmarkRow};
use crate::error::LogiscoreError;

const ACCURACY_EPSILON: f64 = 0.000_001;

#[derive(Debug, Serialize)]
pub struct RegressionComparison {
    pub baseline_schema_version: u8,
    pub fixture_bytes: usize,
    pub functional_regression: bool,
    pub rows: Vec<RegressionRow>,
}

#[derive(Debug, Serialize)]
pub struct RegressionRow {
    pub name: String,
    pub functional_regression: bool,
    pub final_recovery_delta_percent: f64,
    pub raw_symbol_accuracy_delta_percent: f64,
    pub corrected_errors_delta: f64,
    pub payload_bitrate_delta_percent: Option<f64>,
    pub decode_time_delta_percent: Option<f64>,
    pub playback_duration_delta_percent: Option<f64>,
}

pub fn compare_reports(
    baseline: &BenchmarkReport,
    current: &BenchmarkReport,
) -> Result<RegressionComparison, LogiscoreError> {
    validate_compatible(baseline, current)?;
    let rows = baseline
        .rows
        .iter()
        .map(|baseline_row| {
            let current_row = current
                .rows
                .iter()
                .find(|row| row.name == baseline_row.name)
                .ok_or_else(|| invalid_report("current report is missing a benchmark row"))?;
            Ok(compare_row(baseline_row, current_row))
        })
        .collect::<Result<Vec<_>, LogiscoreError>>()?;
    Ok(RegressionComparison {
        baseline_schema_version: baseline.schema_version,
        fixture_bytes: baseline.fixture_bytes,
        functional_regression: rows.iter().any(|row| row.functional_regression),
        rows,
    })
}

fn validate_compatible(
    baseline: &BenchmarkReport,
    current: &BenchmarkReport,
) -> Result<(), LogiscoreError> {
    if baseline.schema_version != current.schema_version {
        return Err(invalid_report("benchmark schema versions differ"));
    }
    if baseline.fixture_bytes != current.fixture_bytes {
        return Err(invalid_report("benchmark fixture sizes differ"));
    }
    if baseline.rows.len() != current.rows.len() {
        return Err(invalid_report("benchmark row counts differ"));
    }
    Ok(())
}

fn compare_row(baseline: &BenchmarkRow, current: &BenchmarkRow) -> RegressionRow {
    let functional_regression = (baseline.clean_roundtrip && !current.clean_roundtrip)
        || current.final_recovery_rate_percent + ACCURACY_EPSILON
            < baseline.final_recovery_rate_percent
        || current.raw_symbol_accuracy_percent + ACCURACY_EPSILON
            < baseline.raw_symbol_accuracy_percent;
    RegressionRow {
        name: baseline.name.clone(),
        functional_regression,
        final_recovery_delta_percent: current.final_recovery_rate_percent
            - baseline.final_recovery_rate_percent,
        raw_symbol_accuracy_delta_percent: current.raw_symbol_accuracy_percent
            - baseline.raw_symbol_accuracy_percent,
        corrected_errors_delta: current.corrected_errors as f64 - baseline.corrected_errors as f64,
        payload_bitrate_delta_percent: optional_delta(
            baseline.payload_bitrate_bps,
            current.payload_bitrate_bps,
        ),
        decode_time_delta_percent: relative_delta(
            baseline.decode_median_micros as f64,
            current.decode_median_micros as f64,
        ),
        playback_duration_delta_percent: optional_delta(
            baseline.playback_ms.map(|value| value as f64),
            current.playback_ms.map(|value| value as f64),
        ),
    }
}

fn optional_delta(baseline: Option<f64>, current: Option<f64>) -> Option<f64> {
    baseline
        .zip(current)
        .and_then(|(baseline, current)| relative_delta(baseline, current))
}

fn relative_delta(baseline: f64, current: f64) -> Option<f64> {
    (baseline != 0.0).then_some((current - baseline) * 100.0 / baseline)
}

fn invalid_report(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidProfile(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_functional_accuracy_regressions_but_not_timing_changes() {
        let baseline = report(row(true, 80.0, 90.0, 100));
        let current = report(row(true, 60.0, 90.0, 200));
        let comparison = compare_reports(&baseline, &current).unwrap();
        assert!(comparison.functional_regression);
        assert_eq!(comparison.rows[0].final_recovery_delta_percent, -20.0);
        assert_eq!(comparison.rows[0].decode_time_delta_percent, Some(100.0));
    }

    #[test]
    fn rejects_different_fixtures() {
        let baseline = report(row(true, 100.0, 100.0, 100));
        let mut current = report(row(true, 100.0, 100.0, 100));
        current.fixture_bytes += 1;
        assert!(compare_reports(&baseline, &current).is_err());
    }

    fn report(row: BenchmarkRow) -> BenchmarkReport {
        BenchmarkReport {
            schema_version: 2,
            fixture_bytes: 32,
            iterations: 1,
            rows: vec![row],
        }
    }

    fn row(
        clean_roundtrip: bool,
        final_recovery_rate_percent: f64,
        raw_symbol_accuracy_percent: f64,
        decode_median_micros: u128,
    ) -> BenchmarkRow {
        BenchmarkRow {
            name: "fixture".to_owned(),
            transport: "WAV".to_owned(),
            output_bytes: 1,
            encode_median_micros: 1,
            decode_median_micros,
            playback_ms: Some(1),
            payload_bitrate_bps: Some(1.0),
            clean_roundtrip,
            final_recovery_rate_percent,
            raw_symbol_accuracy_percent,
            corrected_errors: 0,
            music_weight: None,
            max_polyphony: None,
            fec_profile: None,
            channel_successes: Some(1),
            channel_cases: Some(1),
        }
    }
}
