mod calibration;
mod environment;
mod estimate;
mod profile;
mod selector;

pub use environment::{CalibrationMetrics, Environment};
pub use estimate::{estimate_duration_ms, estimate_payload_duration_ms};
pub use profile::{AcousticProfile, AcousticProfileId};
pub use selector::{select_profile, ProfileSelection, SelectionInput};

#[cfg(test)]
mod tests;
pub use calibration::analyze_calibration;
