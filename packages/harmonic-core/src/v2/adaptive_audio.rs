use crate::adaptive::{
    estimate_duration_ms, select_profile, AcousticProfile, AcousticProfileId, Environment,
    ProfileSelection, SelectionInput,
};
use crate::audio::{encode_wav, FixedPcmCodec, PcmAudio, PcmCodec, PcmProfile};
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::{ProjectFilePayload, ProjectPayload};
use crate::v2_packet::{
    build_packet_for_fec_and_flags, FIXED_AUDIO_CODEC_PROFILE, RHYTHMIC_CODEC_PROFILE,
};

#[derive(Debug, Clone)]
pub struct AdaptiveWav {
    pub bytes: Vec<u8>,
    pub selection: ProfileSelection,
    pub estimated_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AdaptiveDecoded<T> {
    pub payload: T,
    pub profile_id: AcousticProfileId,
}

pub fn encode_text_wav_adaptive(
    text: &str,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    encode_wav_payload(
        PayloadType::Text,
        &TextPayload::new(text).into_bytes(),
        environment,
        reliability_priority,
    )
}

pub fn encode_source_file_wav_adaptive(
    filename: &str,
    extension: &str,
    source: &str,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_wav_payload(
        PayloadType::SourceFile,
        &payload.into_bytes(),
        environment,
        reliability_priority,
    )
}

pub fn encode_project_wav_adaptive(
    files: Vec<ProjectFilePayload>,
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    let payload = ProjectPayload::new(files)?;
    encode_wav_payload(
        PayloadType::Project,
        &payload.into_bytes(),
        environment,
        reliability_priority,
    )
}

pub fn encode_wav_with_profile(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    profile_id: AcousticProfileId,
) -> Result<Vec<u8>, LogiscoreError> {
    let profile = AcousticProfile::for_id(profile_id);
    let packet = build_profile_packet(payload_type, canonical_payload, profile)?;
    let samples = encode_packet_samples(&packet, profile)?;
    encode_wav(&PcmAudio::new(
        PcmProfile::default().sample_rate(),
        samples,
    )?)
}

fn encode_wav_payload(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    environment: Environment,
    reliability_priority: u8,
) -> Result<AdaptiveWav, LogiscoreError> {
    let selection = select_profile(SelectionInput {
        environment,
        reliability_priority,
        payload_bytes: canonical_payload.len(),
        calibration: None,
    })?;
    let packet = build_profile_packet(payload_type, canonical_payload, selection.primary)?;
    let estimated_duration_ms = estimate_duration_ms(selection.primary, packet.len());
    let samples = encode_packet_samples(&packet, selection.primary)?;
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

fn build_profile_packet(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    profile: AcousticProfile,
) -> Result<Vec<u8>, LogiscoreError> {
    let codec_profile = if profile.id == AcousticProfileId::FixedFallback {
        FIXED_AUDIO_CODEC_PROFILE
    } else {
        RHYTHMIC_CODEC_PROFILE
    };
    build_packet_for_fec_and_flags(
        payload_type,
        canonical_payload,
        codec_profile,
        profile.fec_profile,
        profile.id as u8,
    )
}

pub(super) fn encode_packet_samples(
    packet: &[u8],
    profile: AcousticProfile,
) -> Result<Vec<f32>, LogiscoreError> {
    let pcm_profile = PcmProfile::with_timing_percent(profile.timing_percent)?;
    if profile.id == AcousticProfileId::FixedFallback {
        FixedPcmCodec::new(pcm_profile).encode(packet)
    } else {
        PcmCodec::with_profile(pcm_profile).encode(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_adaptive_profile_roundtrips_and_reports_wire_id() {
        for id in [
            AcousticProfileId::Quiet,
            AcousticProfileId::Balanced,
            AcousticProfileId::Conversation,
            AcousticProfileId::Noisy,
            AcousticProfileId::Online,
            AcousticProfileId::LongDistance,
            AcousticProfileId::FixedFallback,
        ] {
            let wav = encode_wav_with_profile(PayloadType::Text, b"profile roundtrip", id).unwrap();
            let decoded = super::super::decode_text_wav_adaptive(&wav).unwrap();
            assert_eq!(decoded.payload, "profile roundtrip");
            assert_eq!(decoded.profile_id, id);
        }
    }

    #[test]
    fn adaptive_decoder_keeps_legacy_reliable_wav_compatible() {
        let legacy = super::super::encode_text_wav_reliable("legacy balanced").unwrap();
        let decoded = super::super::decode_text_wav_adaptive(&legacy).unwrap();
        assert_eq!(decoded.payload, "legacy balanced");
        assert_eq!(decoded.profile_id, AcousticProfileId::Balanced);
    }

    #[test]
    fn explicit_environment_changes_timing_and_selection() {
        let quiet = encode_text_wav_adaptive("timing", Environment::Quiet, 50).unwrap();
        let distant = encode_text_wav_adaptive("timing", Environment::LongDistance, 50).unwrap();
        assert_eq!(quiet.selection.primary.id, AcousticProfileId::Quiet);
        assert_eq!(
            distant.selection.primary.id,
            AcousticProfileId::LongDistance
        );
        assert!(distant.bytes.len() > quiet.bytes.len());
        assert!(distant.estimated_duration_ms > quiet.estimated_duration_ms);
    }
}
