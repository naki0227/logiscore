use super::{AcousticProfile, AcousticProfileId, CalibrationMetrics, Environment};
use crate::error::LogiscoreError;

#[derive(Debug, Clone, Copy)]
pub struct SelectionInput {
    pub environment: Environment,
    pub reliability_priority: u8,
    pub payload_bytes: usize,
    pub calibration: Option<CalibrationMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSelection {
    pub primary: AcousticProfile,
    pub fallback: Vec<AcousticProfileId>,
    pub confidence_percent: u8,
    pub detected_environment: Environment,
}

pub fn select_profile(input: SelectionInput) -> Result<ProfileSelection, LogiscoreError> {
    if input.reliability_priority > 100 {
        return Err(LogiscoreError::InvalidProfile(
            "reliability priority exceeds 100".into(),
        ));
    }
    let (detected, mut confidence) = resolve_environment(input.environment, input.calibration);
    let mut primary_id = profile_for_environment(detected);
    if input.reliability_priority >= 75 && primary_id == AcousticProfileId::Quiet {
        primary_id = AcousticProfileId::Balanced;
        confidence = confidence.saturating_sub(5);
    } else if input.reliability_priority <= 25
        && input.payload_bytes > 512
        && primary_id == AcousticProfileId::Balanced
    {
        primary_id = AcousticProfileId::Quiet;
        confidence = confidence.saturating_sub(10);
    }
    Ok(ProfileSelection {
        primary: AcousticProfile::for_id(primary_id),
        fallback: fallback_order(primary_id),
        confidence_percent: confidence,
        detected_environment: detected,
    })
}

fn resolve_environment(
    requested: Environment,
    calibration: Option<CalibrationMetrics>,
) -> (Environment, u8) {
    if requested != Environment::Auto {
        return (requested, 100);
    }
    let Some(metrics) = calibration else {
        return (Environment::Auto, 50);
    };
    if metrics.clipping_ratio >= 0.02 || metrics.snr_db < 8.0 {
        (Environment::Noisy, 90)
    } else if metrics.reverberation_ms >= 180 {
        (Environment::LongDistance, 85)
    } else if metrics.snr_db < 16.0 || metrics.noise_floor_db > -35.0 {
        (Environment::Conversation, 80)
    } else {
        (Environment::Quiet, 90)
    }
}

fn profile_for_environment(environment: Environment) -> AcousticProfileId {
    match environment {
        Environment::Auto => AcousticProfileId::Balanced,
        Environment::Quiet => AcousticProfileId::Quiet,
        Environment::Conversation => AcousticProfileId::Conversation,
        Environment::Noisy => AcousticProfileId::Noisy,
        Environment::Online => AcousticProfileId::Online,
        Environment::LongDistance => AcousticProfileId::LongDistance,
    }
}

fn fallback_order(primary: AcousticProfileId) -> Vec<AcousticProfileId> {
    use AcousticProfileId::*;
    match primary {
        Quiet => vec![Balanced, Noisy, FixedFallback],
        Balanced => vec![Noisy, LongDistance, FixedFallback],
        Conversation => vec![Noisy, LongDistance, FixedFallback],
        Noisy => vec![LongDistance, FixedFallback],
        Online => vec![Noisy, FixedFallback],
        LongDistance => vec![Noisy, FixedFallback],
        FixedFallback => Vec::new(),
    }
}
