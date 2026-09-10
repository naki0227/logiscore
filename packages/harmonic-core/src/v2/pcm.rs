use crate::audio::PcmCodec;
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::{ProjectFilePayload, ProjectPayload};
use crate::v2_packet::{
    build_packet_for_fec, decode_packet_bytes_for_fec, RELIABLE_FEC_PROFILE, RHYTHMIC_CODEC_PROFILE,
};

const NO_FEC_PROFILE: u8 = 0;

pub fn encode_text_pcm(text: &str) -> Result<Vec<f32>, LogiscoreError> {
    encode_pcm(
        PayloadType::Text,
        &TextPayload::new(text).into_bytes(),
        NO_FEC_PROFILE,
    )
}

pub fn decode_text_pcm(samples: &[f32]) -> Result<String, LogiscoreError> {
    let bytes = decode_pcm(samples, PayloadType::Text, NO_FEC_PROFILE)?;
    TextPayload::from_bytes(bytes).map(|payload| payload.as_str().to_owned())
}

pub fn encode_text_pcm_reliable(text: &str) -> Result<Vec<f32>, LogiscoreError> {
    encode_pcm(
        PayloadType::Text,
        &TextPayload::new(text).into_bytes(),
        RELIABLE_FEC_PROFILE,
    )
}

pub fn decode_text_pcm_reliable(samples: &[f32]) -> Result<String, LogiscoreError> {
    let bytes = decode_pcm(samples, PayloadType::Text, RELIABLE_FEC_PROFILE)?;
    TextPayload::from_bytes(bytes).map(|payload| payload.as_str().to_owned())
}

pub fn decode_text_pcm_reliable_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, LogiscoreError> {
    let bytes = decode_pcm_at_sample_rate(
        samples,
        sample_rate,
        PayloadType::Text,
        RELIABLE_FEC_PROFILE,
    )?;
    TextPayload::from_bytes(bytes).map(|payload| payload.as_str().to_owned())
}

pub fn encode_source_file_pcm(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<f32>, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_pcm(
        PayloadType::SourceFile,
        &payload.into_bytes(),
        NO_FEC_PROFILE,
    )
}

pub fn decode_source_file_pcm(samples: &[f32]) -> Result<SourceFilePayload, LogiscoreError> {
    let bytes = decode_pcm(samples, PayloadType::SourceFile, NO_FEC_PROFILE)?;
    SourceFilePayload::from_bytes(&bytes)
}

pub fn encode_source_file_pcm_reliable(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<f32>, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_pcm(
        PayloadType::SourceFile,
        &payload.into_bytes(),
        RELIABLE_FEC_PROFILE,
    )
}

pub fn decode_source_file_pcm_reliable(
    samples: &[f32],
) -> Result<SourceFilePayload, LogiscoreError> {
    let bytes = decode_pcm(samples, PayloadType::SourceFile, RELIABLE_FEC_PROFILE)?;
    SourceFilePayload::from_bytes(&bytes)
}

pub fn decode_source_file_pcm_reliable_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
) -> Result<SourceFilePayload, LogiscoreError> {
    let bytes = decode_pcm_at_sample_rate(
        samples,
        sample_rate,
        PayloadType::SourceFile,
        RELIABLE_FEC_PROFILE,
    )?;
    SourceFilePayload::from_bytes(&bytes)
}

pub fn encode_project_pcm(files: Vec<ProjectFilePayload>) -> Result<Vec<f32>, LogiscoreError> {
    let payload = ProjectPayload::new(files)?;
    encode_pcm(PayloadType::Project, &payload.into_bytes(), NO_FEC_PROFILE)
}

pub fn decode_project_pcm(samples: &[f32]) -> Result<ProjectPayload, LogiscoreError> {
    let bytes = decode_pcm(samples, PayloadType::Project, NO_FEC_PROFILE)?;
    ProjectPayload::from_bytes(&bytes)
}

pub fn encode_project_pcm_reliable(
    files: Vec<ProjectFilePayload>,
) -> Result<Vec<f32>, LogiscoreError> {
    let payload = ProjectPayload::new(files)?;
    encode_pcm(
        PayloadType::Project,
        &payload.into_bytes(),
        RELIABLE_FEC_PROFILE,
    )
}

pub fn decode_project_pcm_reliable(samples: &[f32]) -> Result<ProjectPayload, LogiscoreError> {
    let bytes = decode_pcm(samples, PayloadType::Project, RELIABLE_FEC_PROFILE)?;
    ProjectPayload::from_bytes(&bytes)
}

pub fn decode_project_pcm_reliable_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
) -> Result<ProjectPayload, LogiscoreError> {
    let bytes = decode_pcm_at_sample_rate(
        samples,
        sample_rate,
        PayloadType::Project,
        RELIABLE_FEC_PROFILE,
    )?;
    ProjectPayload::from_bytes(&bytes)
}

pub fn pcm_sample_rate() -> u32 {
    PcmCodec::default().sample_rate()
}

fn encode_pcm(
    payload_type: PayloadType,
    canonical_payload: &[u8],
    fec_profile: u8,
) -> Result<Vec<f32>, LogiscoreError> {
    let packet = build_packet_for_fec(
        payload_type,
        canonical_payload,
        RHYTHMIC_CODEC_PROFILE,
        fec_profile,
    )?;
    PcmCodec::default().encode(&packet)
}

fn decode_pcm(
    samples: &[f32],
    payload_type: PayloadType,
    fec_profile: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    decode_pcm_at_sample_rate(samples, pcm_sample_rate(), payload_type, fec_profile)
}

fn decode_pcm_at_sample_rate(
    samples: &[f32],
    sample_rate: u32,
    payload_type: PayloadType,
    fec_profile: u8,
) -> Result<Vec<u8>, LogiscoreError> {
    let packet = PcmCodec::default().decode_at_sample_rate(samples, sample_rate)?;
    decode_packet_bytes_for_fec(&packet, payload_type, RHYTHMIC_CODEC_PROFILE, fec_profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_pcm_roundtrips_unicode() {
        let text = "PCMからこんにちは 🎧";
        let samples = encode_text_pcm(text).unwrap();
        assert_eq!(decode_text_pcm(&samples).unwrap(), text);
    }

    #[test]
    fn source_file_pcm_roundtrips_metadata() {
        let samples = encode_source_file_pcm("main", ".rs", "fn main() {}").unwrap();
        let decoded = decode_source_file_pcm(&samples).unwrap();
        assert_eq!(decoded.filename(), "main");
        assert_eq!(decoded.extension(), ".rs");
        assert_eq!(decoded.source(), "fn main() {}");
    }

    #[test]
    fn project_pcm_roundtrips_files() {
        let files = vec![
            ProjectFilePayload::new("README.md", ".md", "# PCM").unwrap(),
            ProjectFilePayload::new("src/main.rs", ".rs", "fn main() {}").unwrap(),
        ];
        let samples = encode_project_pcm(files.clone()).unwrap();
        assert_eq!(
            decode_project_pcm(&samples).unwrap().files(),
            ProjectPayload::new(files).unwrap().files()
        );
    }

    #[test]
    fn pcm_decoder_rejects_wrong_payload_type() {
        let samples = encode_text_pcm("not a project").unwrap();
        assert!(decode_project_pcm(&samples).is_err());
    }

    #[test]
    fn reliable_pcm_roundtrips_all_payload_types() {
        let text = encode_text_pcm_reliable("reliable 🎧").unwrap();
        assert_eq!(decode_text_pcm_reliable(&text).unwrap(), "reliable 🎧");

        let source = encode_source_file_pcm_reliable("main.rs", ".rs", "fn main() {}").unwrap();
        assert_eq!(
            decode_source_file_pcm_reliable(&source).unwrap().source(),
            "fn main() {}"
        );

        let project_files = vec![ProjectFilePayload::new("README.md", ".md", "# FEC").unwrap()];
        let project = encode_project_pcm_reliable(project_files).unwrap();
        assert_eq!(
            decode_project_pcm_reliable(&project).unwrap().files().len(),
            1
        );
    }

    #[test]
    fn reliable_and_unprotected_pcm_profiles_are_not_interchangeable() {
        let reliable = encode_text_pcm_reliable("profile").unwrap();
        assert!(decode_text_pcm(&reliable).is_err());
        let unprotected = encode_text_pcm("profile").unwrap();
        assert!(decode_text_pcm_reliable(&unprotected).is_err());
    }

    #[test]
    fn reliable_pcm_recovers_a_corrupted_packet_body() {
        let expected = "correct through PCM";
        let canonical = TextPayload::new(expected).into_bytes();
        let mut packet = build_packet_for_fec(
            PayloadType::Text,
            &canonical,
            RHYTHMIC_CODEC_PROFILE,
            RELIABLE_FEC_PROFILE,
        )
        .unwrap();
        let body_start = crate::protocol::v2::HEADER_LENGTH;
        let copy_length = (packet.len() - body_start) / 3;
        packet[body_start] ^= 0x80;
        packet[body_start + copy_length] ^= 0x80;

        let samples = PcmCodec::default().encode(&packet).unwrap();
        assert_eq!(decode_text_pcm_reliable(&samples).unwrap(), expected);
    }
}
