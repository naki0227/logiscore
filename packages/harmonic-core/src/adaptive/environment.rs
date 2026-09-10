use crate::error::LogiscoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Auto,
    Quiet,
    Conversation,
    Noisy,
    Online,
    LongDistance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationMetrics {
    pub noise_floor_db: f32,
    pub snr_db: f32,
    pub clipping_ratio: f32,
    pub reverberation_ms: u16,
}

impl CalibrationMetrics {
    pub fn new(
        noise_floor_db: f32,
        snr_db: f32,
        clipping_ratio: f32,
        reverberation_ms: u16,
    ) -> Result<Self, LogiscoreError> {
        if !noise_floor_db.is_finite()
            || !snr_db.is_finite()
            || !clipping_ratio.is_finite()
            || !(-120.0..=0.0).contains(&noise_floor_db)
            || !(-20.0..=120.0).contains(&snr_db)
            || !(0.0..=1.0).contains(&clipping_ratio)
            || reverberation_ms > 5_000
        {
            return Err(LogiscoreError::InvalidProfile(
                "calibration metrics are outside supported ranges".into(),
            ));
        }
        Ok(Self {
            noise_floor_db,
            snr_db,
            clipping_ratio,
            reverberation_ms,
        })
    }
}
