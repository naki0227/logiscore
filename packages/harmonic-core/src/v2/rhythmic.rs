use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::{ProjectFilePayload, ProjectPayload};
use crate::transport::rhythmic_midi::RhythmicMidiTransport;
use crate::v2_packet::{
    decode_packet_for_profile, encode_packet_for_profile, RHYTHMIC_CODEC_PROFILE,
};

pub fn encode_text_rhythmic(text: &str) -> Result<Vec<u8>, LogiscoreError> {
    encode_packet_for_profile(
        PayloadType::Text,
        &TextPayload::new(text).into_bytes(),
        RHYTHMIC_CODEC_PROFILE,
        &RhythmicMidiTransport::default(),
    )
}

pub fn decode_text_rhythmic(midi_bytes: &[u8]) -> Result<String, LogiscoreError> {
    let bytes = decode_packet_for_profile(
        midi_bytes,
        PayloadType::Text,
        RHYTHMIC_CODEC_PROFILE,
        &RhythmicMidiTransport::default(),
    )?;
    TextPayload::from_bytes(bytes).map(|payload| payload.as_str().to_owned())
}

pub fn encode_source_file_rhythmic(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_packet_for_profile(
        PayloadType::SourceFile,
        &payload.into_bytes(),
        RHYTHMIC_CODEC_PROFILE,
        &RhythmicMidiTransport::default(),
    )
}

pub fn decode_source_file_rhythmic(midi_bytes: &[u8]) -> Result<SourceFilePayload, LogiscoreError> {
    let bytes = decode_packet_for_profile(
        midi_bytes,
        PayloadType::SourceFile,
        RHYTHMIC_CODEC_PROFILE,
        &RhythmicMidiTransport::default(),
    )?;
    SourceFilePayload::from_bytes(&bytes)
}

pub fn encode_project_rhythmic(files: Vec<ProjectFilePayload>) -> Result<Vec<u8>, LogiscoreError> {
    let payload = ProjectPayload::new(files)?;
    encode_packet_for_profile(
        PayloadType::Project,
        &payload.into_bytes(),
        RHYTHMIC_CODEC_PROFILE,
        &RhythmicMidiTransport::default(),
    )
}

pub fn decode_project_rhythmic(midi_bytes: &[u8]) -> Result<ProjectPayload, LogiscoreError> {
    let bytes = decode_packet_for_profile(
        midi_bytes,
        PayloadType::Project,
        RHYTHMIC_CODEC_PROFILE,
        &RhythmicMidiTransport::default(),
    )?;
    ProjectPayload::from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::v2::BinaryHeader;
    use crate::transport::Transport;

    #[test]
    fn text_rhythmic_roundtrips_unicode_and_declares_profile_two() {
        let text = "Rhythm codecからこんにちは 🎶";
        let midi = encode_text_rhythmic(text).unwrap();
        assert_eq!(decode_text_rhythmic(&midi).unwrap(), text);
        let packet = RhythmicMidiTransport::default().decode(&midi).unwrap();
        assert_eq!(
            BinaryHeader::decode(&packet).unwrap().codec_profile,
            RHYTHMIC_CODEC_PROFILE
        );
    }

    #[test]
    fn source_file_rhythmic_roundtrips_metadata() {
        let midi = encode_source_file_rhythmic("main", ".rs", "fn main() {}").unwrap();
        let decoded = decode_source_file_rhythmic(&midi).unwrap();
        assert_eq!(decoded.filename(), "main");
        assert_eq!(decoded.extension(), ".rs");
        assert_eq!(decoded.source(), "fn main() {}");
    }

    #[test]
    fn project_rhythmic_roundtrips_files() {
        let files = vec![
            ProjectFilePayload::new("README.md", ".md", "# Rhythm").unwrap(),
            ProjectFilePayload::new("src/main.rs", ".rs", "fn main() {}").unwrap(),
        ];
        let midi = encode_project_rhythmic(files.clone()).unwrap();
        assert_eq!(
            decode_project_rhythmic(&midi).unwrap().files(),
            ProjectPayload::new(files).unwrap().files()
        );
    }
}
