use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::{ProjectFilePayload, ProjectPayload};
use crate::transport::dense_midi::DenseMidiTransport;
use crate::transport::musical_midi::MusicalMidiTransport;
use crate::transport::Transport;
use crate::v2_packet::{
    decode_packet, decode_packet_for_profile, encode_packet, encode_packet_for_profile,
    MUSICAL_CODEC_PROFILE,
};

mod adaptive_audio;
pub use adaptive_audio::*;
mod adaptive_decode;
pub use adaptive_decode::*;
mod secure_audio;
pub use secure_audio::*;
mod secure_decode;
pub use secure_decode::*;
mod rhythmic;
pub use rhythmic::*;
mod pcm;
pub use pcm::*;
mod wav;
pub use wav::*;

pub fn encode_text<T: Transport>(
    payload: TextPayload,
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    encode_packet(PayloadType::Text, &payload.into_bytes(), transport)
}

pub fn decode_text<T: Transport>(
    transport_data: &[u8],
    transport: &T,
) -> Result<TextPayload, LogiscoreError> {
    TextPayload::from_bytes(decode_packet(transport_data, PayloadType::Text, transport)?)
}

pub fn encode_source_file<T: Transport>(
    payload: SourceFilePayload,
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    encode_packet(PayloadType::SourceFile, &payload.into_bytes(), transport)
}

pub fn decode_source_file<T: Transport>(
    transport_data: &[u8],
    transport: &T,
) -> Result<SourceFilePayload, LogiscoreError> {
    SourceFilePayload::from_bytes(&decode_packet(
        transport_data,
        PayloadType::SourceFile,
        transport,
    )?)
}

pub fn encode_project<T: Transport>(
    payload: ProjectPayload,
    transport: &T,
) -> Result<Vec<u8>, LogiscoreError> {
    encode_packet(PayloadType::Project, &payload.into_bytes(), transport)
}

pub fn decode_project<T: Transport>(
    transport_data: &[u8],
    transport: &T,
) -> Result<ProjectPayload, LogiscoreError> {
    ProjectPayload::from_bytes(&decode_packet(
        transport_data,
        PayloadType::Project,
        transport,
    )?)
}

pub fn encode_text_dense(text: &str) -> Result<Vec<u8>, LogiscoreError> {
    encode_text(TextPayload::new(text), &DenseMidiTransport::default())
}

pub fn decode_text_dense(midi_bytes: &[u8]) -> Result<String, LogiscoreError> {
    decode_text(midi_bytes, &DenseMidiTransport::default())
        .map(|payload| payload.as_str().to_owned())
}

pub fn encode_source_file_dense(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_source_file(payload, &DenseMidiTransport::default())
}

pub fn decode_source_file_dense(midi_bytes: &[u8]) -> Result<SourceFilePayload, LogiscoreError> {
    decode_source_file(midi_bytes, &DenseMidiTransport::default())
}

pub fn encode_project_dense(files: Vec<ProjectFilePayload>) -> Result<Vec<u8>, LogiscoreError> {
    encode_project(ProjectPayload::new(files)?, &DenseMidiTransport::default())
}

pub fn decode_project_dense(midi_bytes: &[u8]) -> Result<ProjectPayload, LogiscoreError> {
    decode_project(midi_bytes, &DenseMidiTransport::default())
}

pub fn encode_text_musical(text: &str) -> Result<Vec<u8>, LogiscoreError> {
    encode_packet_for_profile(
        PayloadType::Text,
        &TextPayload::new(text).into_bytes(),
        MUSICAL_CODEC_PROFILE,
        &MusicalMidiTransport::default(),
    )
}

pub fn decode_text_musical(midi_bytes: &[u8]) -> Result<String, LogiscoreError> {
    let bytes = decode_packet_for_profile(
        midi_bytes,
        PayloadType::Text,
        MUSICAL_CODEC_PROFILE,
        &MusicalMidiTransport::default(),
    )?;
    TextPayload::from_bytes(bytes).map(|payload| payload.as_str().to_owned())
}

pub fn encode_source_file_musical(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, LogiscoreError> {
    let payload = SourceFilePayload::new(filename, extension, source)?;
    encode_packet_for_profile(
        PayloadType::SourceFile,
        &payload.into_bytes(),
        MUSICAL_CODEC_PROFILE,
        &MusicalMidiTransport::default(),
    )
}

pub fn decode_source_file_musical(midi_bytes: &[u8]) -> Result<SourceFilePayload, LogiscoreError> {
    let bytes = decode_packet_for_profile(
        midi_bytes,
        PayloadType::SourceFile,
        MUSICAL_CODEC_PROFILE,
        &MusicalMidiTransport::default(),
    )?;
    SourceFilePayload::from_bytes(&bytes)
}

pub fn encode_project_musical(files: Vec<ProjectFilePayload>) -> Result<Vec<u8>, LogiscoreError> {
    let payload = ProjectPayload::new(files)?;
    encode_packet_for_profile(
        PayloadType::Project,
        &payload.into_bytes(),
        MUSICAL_CODEC_PROFILE,
        &MusicalMidiTransport::default(),
    )
}

pub fn decode_project_musical(midi_bytes: &[u8]) -> Result<ProjectPayload, LogiscoreError> {
    let bytes = decode_packet_for_profile(
        midi_bytes,
        PayloadType::Project,
        MUSICAL_CODEC_PROFILE,
        &MusicalMidiTransport::default(),
    )?;
    ProjectPayload::from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_dense_roundtrips_empty_text() {
        let midi = encode_text_dense("").unwrap();
        assert_eq!(decode_text_dense(&midi).unwrap(), "");
    }

    #[test]
    fn text_dense_roundtrips_unicode() {
        let text = "Hello from Logiscore. 明日13時集合 🎼";
        let midi = encode_text_dense(text).unwrap();
        assert_eq!(decode_text_dense(&midi).unwrap(), text);
    }

    #[test]
    fn text_dense_encoding_is_deterministic() {
        let text = "same input, same MIDI";
        assert_eq!(
            encode_text_dense(text).unwrap(),
            encode_text_dense(text).unwrap()
        );
    }

    #[test]
    fn source_file_dense_roundtrips_metadata_and_utf8() {
        let midi = encode_source_file_dense("挨拶", ".rs", "fn main() { /* 🎼 */ }").unwrap();
        let decoded = decode_source_file_dense(&midi).unwrap();
        assert_eq!(decoded.filename(), "挨拶");
        assert_eq!(decoded.extension(), ".rs");
        assert_eq!(decoded.source(), "fn main() { /* 🎼 */ }");
    }

    #[test]
    fn source_file_dense_encoding_is_deterministic() {
        let encode = || encode_source_file_dense("main", ".rs", "fn main() {}").unwrap();
        assert_eq!(encode(), encode());
    }

    #[test]
    fn source_file_decoder_rejects_text_payload() {
        let midi = encode_text_dense("not a source file").unwrap();
        assert!(decode_source_file_dense(&midi).is_err());
    }

    #[test]
    fn project_dense_roundtrips_nested_files() {
        let files = vec![
            ProjectFilePayload::new("src/main.rs", ".rs", "fn main() {}").unwrap(),
            ProjectFilePayload::new("README.md", ".md", "# Project 🎼").unwrap(),
        ];
        let midi = encode_project_dense(files.clone()).unwrap();
        assert_eq!(
            decode_project_dense(&midi).unwrap().files(),
            ProjectPayload::new(files).unwrap().files()
        );
    }

    #[test]
    fn project_decoder_rejects_source_file_payload() {
        let midi = encode_source_file_dense("main", ".rs", "source").unwrap();
        assert!(decode_project_dense(&midi).is_err());
    }

    #[test]
    fn text_musical_roundtrips_unicode() {
        let text = "Musical codecからこんにちは 🎼";
        let midi = encode_text_musical(text).unwrap();
        assert_eq!(decode_text_musical(&midi).unwrap(), text);
    }

    #[test]
    fn source_file_musical_roundtrips_metadata() {
        let midi = encode_source_file_musical("main", ".rs", "fn main() {}").unwrap();
        let decoded = decode_source_file_musical(&midi).unwrap();
        assert_eq!(decoded.filename(), "main");
        assert_eq!(decoded.extension(), ".rs");
        assert_eq!(decoded.source(), "fn main() {}");
    }

    #[test]
    fn project_musical_roundtrips_files() {
        let files = vec![
            ProjectFilePayload::new("README.md", ".md", "# Music").unwrap(),
            ProjectFilePayload::new("src/main.rs", ".rs", "fn main() {}").unwrap(),
        ];
        let midi = encode_project_musical(files.clone()).unwrap();
        assert_eq!(
            decode_project_musical(&midi).unwrap().files(),
            ProjectPayload::new(files).unwrap().files()
        );
    }

    #[test]
    fn musical_decoder_rejects_dense_transport() {
        let midi = encode_text_dense("wrong transport").unwrap();
        assert!(decode_text_musical(&midi).is_err());
    }

    #[test]
    fn musical_packet_declares_codec_profile_one() {
        let midi = encode_text_musical("profile").unwrap();
        let packet = MusicalMidiTransport::default().decode(&midi).unwrap();
        let header = crate::protocol::v2::BinaryHeader::decode(&packet).unwrap();
        assert_eq!(header.codec_profile, MUSICAL_CODEC_PROFILE);
    }
}
