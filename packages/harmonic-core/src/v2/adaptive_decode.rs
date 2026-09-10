use super::AdaptiveDecoded;
use crate::adaptive::{
    analyze_calibration, select_profile, AcousticProfile, AcousticProfileId, Environment,
    SelectionInput,
};
use crate::audio::{decode_wav, FixedPcmCodec, PcmCodec, PcmProfile};
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::ProjectPayload;
use crate::v2_packet::{
    decode_packet_bytes_for_fec_and_flags, FIXED_AUDIO_CODEC_PROFILE, RELIABLE_FEC_PROFILE,
    RHYTHMIC_CODEC_PROFILE,
};

pub fn decode_text_wav_adaptive(bytes: &[u8]) -> Result<AdaptiveDecoded<String>, LogiscoreError> {
    let decoded = decode_wav_payload(bytes, PayloadType::Text)?;
    Ok(AdaptiveDecoded {
        payload: TextPayload::from_bytes(decoded.payload)?
            .as_str()
            .to_owned(),
        profile_id: decoded.profile_id,
    })
}

pub fn decode_text_pcm_adaptive_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
) -> Result<AdaptiveDecoded<String>, LogiscoreError> {
    let decoded = decode_pcm_payload(samples, sample_rate, PayloadType::Text)?;
    Ok(AdaptiveDecoded {
        payload: TextPayload::from_bytes(decoded.payload)?
            .as_str()
            .to_owned(),
        profile_id: decoded.profile_id,
    })
}

pub fn decode_source_file_wav_adaptive(
    bytes: &[u8],
) -> Result<AdaptiveDecoded<SourceFilePayload>, LogiscoreError> {
    let decoded = decode_wav_payload(bytes, PayloadType::SourceFile)?;
    Ok(AdaptiveDecoded {
        payload: SourceFilePayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

pub fn decode_source_file_pcm_adaptive_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
) -> Result<AdaptiveDecoded<SourceFilePayload>, LogiscoreError> {
    let decoded = decode_pcm_payload(samples, sample_rate, PayloadType::SourceFile)?;
    Ok(AdaptiveDecoded {
        payload: SourceFilePayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

pub fn decode_project_wav_adaptive(
    bytes: &[u8],
) -> Result<AdaptiveDecoded<ProjectPayload>, LogiscoreError> {
    let decoded = decode_wav_payload(bytes, PayloadType::Project)?;
    Ok(AdaptiveDecoded {
        payload: ProjectPayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

pub fn decode_project_pcm_adaptive_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
) -> Result<AdaptiveDecoded<ProjectPayload>, LogiscoreError> {
    let decoded = decode_pcm_payload(samples, sample_rate, PayloadType::Project)?;
    Ok(AdaptiveDecoded {
        payload: ProjectPayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

fn decode_wav_payload(
    bytes: &[u8],
    payload_type: PayloadType,
) -> Result<AdaptiveDecoded<Vec<u8>>, LogiscoreError> {
    let audio = decode_wav(bytes)?;
    decode_pcm_payload(audio.samples(), audio.sample_rate(), payload_type)
}

fn decode_pcm_payload(
    samples: &[f32],
    sample_rate: u32,
    payload_type: PayloadType,
) -> Result<AdaptiveDecoded<Vec<u8>>, LogiscoreError> {
    for profile_id in candidate_profiles(samples, sample_rate)? {
        if let Ok(payload) = try_decode_profile(samples, sample_rate, payload_type, profile_id) {
            return Ok(AdaptiveDecoded {
                payload,
                profile_id,
            });
        }
    }
    try_decode_legacy_balanced(samples, sample_rate, payload_type).map(|payload| AdaptiveDecoded {
        payload,
        profile_id: AcousticProfileId::Balanced,
    })
}

pub(super) fn candidate_profiles(
    samples: &[f32],
    sample_rate: u32,
) -> Result<Vec<AcousticProfileId>, LogiscoreError> {
    let selection = select_profile(SelectionInput {
        environment: Environment::Auto,
        reliability_priority: 50,
        payload_bytes: 0,
        calibration: analyze_calibration(samples, sample_rate).ok(),
    })?;
    Ok(decode_candidates(selection.primary.id, &selection.fallback))
}

fn decode_candidates(
    primary: AcousticProfileId,
    fallback: &[AcousticProfileId],
) -> Vec<AcousticProfileId> {
    let ordered = [
        vec![primary],
        fallback.to_vec(),
        vec![
            AcousticProfileId::Balanced,
            AcousticProfileId::Quiet,
            AcousticProfileId::Conversation,
            AcousticProfileId::Noisy,
            AcousticProfileId::Online,
            AcousticProfileId::LongDistance,
            AcousticProfileId::FixedFallback,
        ],
    ]
    .concat();
    let mut candidates = Vec::new();
    for candidate in ordered {
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn try_decode_profile(
    samples: &[f32],
    sample_rate: u32,
    payload_type: PayloadType,
    profile_id: AcousticProfileId,
) -> Result<Vec<u8>, LogiscoreError> {
    let (packet, codec_profile, profile) = decode_profile_packet(samples, sample_rate, profile_id)?;
    decode_packet_bytes_for_fec_and_flags(
        &packet,
        payload_type,
        codec_profile,
        profile.fec_profile,
        profile_id as u8,
    )
}

pub(crate) fn decode_profile_packet(
    samples: &[f32],
    sample_rate: u32,
    profile_id: AcousticProfileId,
) -> Result<(Vec<u8>, u8, AcousticProfile), LogiscoreError> {
    let profile = AcousticProfile::for_id(profile_id);
    let pcm_profile = PcmProfile::with_timing_percent(profile.timing_percent)?;
    let (packet, codec_profile) = if profile_id == AcousticProfileId::FixedFallback {
        (
            FixedPcmCodec::new(pcm_profile).decode_at_sample_rate(samples, sample_rate)?,
            FIXED_AUDIO_CODEC_PROFILE,
        )
    } else {
        (
            PcmCodec::with_profile(pcm_profile).decode_at_sample_rate(samples, sample_rate)?,
            RHYTHMIC_CODEC_PROFILE,
        )
    };
    Ok((packet, codec_profile, profile))
}

fn try_decode_legacy_balanced(
    samples: &[f32],
    sample_rate: u32,
    payload_type: PayloadType,
) -> Result<Vec<u8>, LogiscoreError> {
    let packet = PcmCodec::default().decode_at_sample_rate(samples, sample_rate)?;
    decode_packet_bytes_for_fec_and_flags(
        &packet,
        payload_type,
        RHYTHMIC_CODEC_PROFILE,
        RELIABLE_FEC_PROFILE,
        0,
    )
}
