use super::adaptive_decode::{candidate_profiles, decode_profile_packet};
use super::AdaptiveDecoded;
use crate::audio::decode_wav;
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::ProjectPayload;
use crate::protocol::v2::BinaryHeader;
use crate::v2_secure_packet::decode_secure_packet_for_fec_and_flags;

#[derive(Debug, Clone)]
pub enum SecurePayload {
    Text(String),
    SourceFile(SourceFilePayload),
    Project(ProjectPayload),
}

#[derive(Debug, Clone)]
pub struct SecureDecoded {
    pub payload: SecurePayload,
    pub profile_id: crate::adaptive::AcousticProfileId,
}

pub fn decode_wav_secure_auto(
    bytes: &[u8],
    password: &str,
) -> Result<SecureDecoded, LogiscoreError> {
    let audio = decode_wav(bytes)?;
    decode_pcm_secure_auto(audio.samples(), audio.sample_rate(), password)
}

pub fn decode_pcm_secure_auto(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<SecureDecoded, LogiscoreError> {
    for profile_id in candidate_profiles(samples, sample_rate)? {
        let Ok((packet, codec_profile, profile)) =
            decode_profile_packet(samples, sample_rate, profile_id)
        else {
            continue;
        };
        let Ok(header) = BinaryHeader::decode(&packet) else {
            continue;
        };
        match decode_secure_packet_for_fec_and_flags(
            &packet,
            password,
            header.payload_type,
            codec_profile,
            profile.fec_profile,
            profile_id as u8,
        ) {
            Ok(payload) => {
                return Ok(SecureDecoded {
                    payload: parse_secure_payload(header.payload_type, payload)?,
                    profile_id,
                })
            }
            Err(LogiscoreError::InvalidPacket(_) | LogiscoreError::InvalidFec(_)) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(LogiscoreError::InvalidAudio(
        "secure Logiscore profile was not detected".into(),
    ))
}

fn parse_secure_payload(
    payload_type: PayloadType,
    payload: Vec<u8>,
) -> Result<SecurePayload, LogiscoreError> {
    match payload_type {
        PayloadType::Text => Ok(SecurePayload::Text(
            TextPayload::from_bytes(payload)?.as_str().to_owned(),
        )),
        PayloadType::SourceFile => Ok(SecurePayload::SourceFile(SourceFilePayload::from_bytes(
            &payload,
        )?)),
        PayloadType::Project => Ok(SecurePayload::Project(ProjectPayload::from_bytes(
            &payload,
        )?)),
    }
}

pub fn decode_text_wav_secure(
    bytes: &[u8],
    password: &str,
) -> Result<AdaptiveDecoded<String>, LogiscoreError> {
    let decoded = decode_secure_wav_payload(bytes, password, PayloadType::Text)?;
    Ok(AdaptiveDecoded {
        payload: TextPayload::from_bytes(decoded.payload)?
            .as_str()
            .to_owned(),
        profile_id: decoded.profile_id,
    })
}

pub fn decode_text_pcm_secure_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<AdaptiveDecoded<String>, LogiscoreError> {
    let decoded = decode_secure_pcm_payload(samples, sample_rate, password, PayloadType::Text)?;
    Ok(AdaptiveDecoded {
        payload: TextPayload::from_bytes(decoded.payload)?
            .as_str()
            .to_owned(),
        profile_id: decoded.profile_id,
    })
}

pub fn decode_source_file_wav_secure(
    bytes: &[u8],
    password: &str,
) -> Result<AdaptiveDecoded<SourceFilePayload>, LogiscoreError> {
    let decoded = decode_secure_wav_payload(bytes, password, PayloadType::SourceFile)?;
    Ok(AdaptiveDecoded {
        payload: SourceFilePayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

pub fn decode_source_file_pcm_secure_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<AdaptiveDecoded<SourceFilePayload>, LogiscoreError> {
    let decoded =
        decode_secure_pcm_payload(samples, sample_rate, password, PayloadType::SourceFile)?;
    Ok(AdaptiveDecoded {
        payload: SourceFilePayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

pub fn decode_project_wav_secure(
    bytes: &[u8],
    password: &str,
) -> Result<AdaptiveDecoded<ProjectPayload>, LogiscoreError> {
    let decoded = decode_secure_wav_payload(bytes, password, PayloadType::Project)?;
    Ok(AdaptiveDecoded {
        payload: ProjectPayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

pub fn decode_project_pcm_secure_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<AdaptiveDecoded<ProjectPayload>, LogiscoreError> {
    let decoded = decode_secure_pcm_payload(samples, sample_rate, password, PayloadType::Project)?;
    Ok(AdaptiveDecoded {
        payload: ProjectPayload::from_bytes(&decoded.payload)?,
        profile_id: decoded.profile_id,
    })
}

fn decode_secure_wav_payload(
    bytes: &[u8],
    password: &str,
    payload_type: PayloadType,
) -> Result<AdaptiveDecoded<Vec<u8>>, LogiscoreError> {
    let audio = decode_wav(bytes)?;
    decode_secure_pcm_payload(audio.samples(), audio.sample_rate(), password, payload_type)
}

fn decode_secure_pcm_payload(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
    payload_type: PayloadType,
) -> Result<AdaptiveDecoded<Vec<u8>>, LogiscoreError> {
    for profile_id in candidate_profiles(samples, sample_rate)? {
        let Ok((packet, codec_profile, profile)) =
            decode_profile_packet(samples, sample_rate, profile_id)
        else {
            continue;
        };
        match decode_secure_packet_for_fec_and_flags(
            &packet,
            password,
            payload_type,
            codec_profile,
            profile.fec_profile,
            profile_id as u8,
        ) {
            Ok(payload) => {
                return Ok(AdaptiveDecoded {
                    payload,
                    profile_id,
                })
            }
            Err(LogiscoreError::InvalidPacket(_) | LogiscoreError::InvalidFec(_)) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(LogiscoreError::InvalidAudio(
        "secure Logiscore profile was not detected".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive::Environment;

    #[test]
    fn secure_wav_roundtrips_all_payload_types_and_rejects_wrong_password() {
        let text = super::super::encode_text_wav_secure(
            "secure text 🎼",
            "shared-password",
            Environment::Quiet,
            50,
        )
        .unwrap();
        assert_eq!(
            decode_text_wav_secure(&text.bytes, "shared-password")
                .unwrap()
                .payload,
            "secure text 🎼"
        );
        assert!(matches!(
            decode_text_wav_secure(&text.bytes, "wrong-password"),
            Err(LogiscoreError::AuthenticationFailed)
        ));

        let source = super::super::encode_source_file_wav_secure(
            "main.rs",
            ".rs",
            "fn main() {}",
            "password",
            Environment::Quiet,
            50,
        )
        .unwrap();
        assert_eq!(
            decode_source_file_wav_secure(&source.bytes, "password")
                .unwrap()
                .payload
                .source(),
            "fn main() {}"
        );

        let files =
            vec![
                crate::project_payload::ProjectFilePayload::new("README.md", ".md", "# Secure")
                    .unwrap(),
            ];
        let project =
            super::super::encode_project_wav_secure(files, "password", Environment::Quiet, 50)
                .unwrap();
        assert_eq!(
            decode_project_wav_secure(&project.bytes, "password")
                .unwrap()
                .payload
                .files()
                .len(),
            1
        );
    }
}
