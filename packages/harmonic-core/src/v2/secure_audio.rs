use super::adaptive_audio::encode_packet_samples;
use super::AdaptiveWav;
use crate::adaptive::{
    estimate_duration_ms, select_profile, AcousticProfileId, Environment, SelectionInput,
};
use crate::audio::{encode_wav, PcmAudio, PcmProfile};
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::{ProjectFilePayload, ProjectPayload};
use crate::v2_packet::{FIXED_AUDIO_CODEC_PROFILE, RHYTHMIC_CODEC_PROFILE};
use crate::v2_secure_packet::build_secure_packet_for_fec_and_flags;

pub fn encode_text_wav_secure(
    text: &str,
    password: &str,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    encode_secure_wav_payload(
        PayloadType::Text,
        &TextPayload::new(text).into_bytes(),
        password,
        environment,
        reliability_priority,
    )
}

pub fn encode_source_file_wav_secure(
    filename: &str,
    extension: &str,
    source: &str,
    password: &str,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_secure_wav_payload(
        PayloadType::SourceFile,
        &payload.into_bytes(),
        password,
        environment,
        reliability_priority,
    )
}

pub fn encode_project_wav_secure(
    files: Vec<ProjectFilePayload>,
    password: &str,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    let payload = ProjectPayload::new(files)?;
    encode_secure_wav_payload(
        PayloadType::Project,
        &payload.into_bytes(),
        password,
        environment,
        reliability_priority,
    )
}

fn encode_secure_wav_payload(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    password: &str,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    let selection = select_profile(SelectionInput {
        environment,
        reliability_priority,
        payload_bytes: canonical_payload.len(),
        calibration: None,
    })?;
    let profile = selection.primary;
    let codec_profile = if profile.id == AcousticProfileId::FixedFallback {
        FIXED_AUDIO_CODEC_PROFILE
    } else {
        RHYTHMIC_CODEC_PROFILE
    };
    let packet = build_secure_packet_for_fec_and_flags(
        payload_type,
        canonical_payload,
        password,
        codec_profile,
        profile.fec_profile,
        profile.id as u8,
    )?;
    let estimated_duration_ms = estimate_duration_ms(profile, packet.len());
    let samples = encode_packet_samples(&packet, profile)?;
    let bytes = encode_wav(&PcmAudio::new(
        PcmProfile::default().sample_rate(),
        samples,
    )?)?;
    Ok(AdaptiveWav {
        bytes,
        selection,
        estimated_duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_audio_uses_randomized_envelopes() {
        let first = encode_text_wav_secure("same", "password", Environment::Quiet, 50).unwrap();
        let second = encode_text_wav_secure("same", "password", Environment::Quiet, 50).unwrap();
        assert_ne!(first.bytes, second.bytes);
    }
}
