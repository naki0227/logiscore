use crate::error::LogiscoreError;

/// v2 payload category encoded in the protocol header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PayloadType {
    Text = 0,
    SourceFile = 1,
    Project = 2,
}

impl TryFrom<u8> for PayloadType {
    type Error = LogiscoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Text),
            1 => Ok(Self::SourceFile),
            2 => Ok(Self::Project),
            _ => Err(LogiscoreError::InvalidPacket(format!(
                "unsupported payload type: {value}"
            ))),
        }
    }
}

/// The first v2 payload implementation. File and project canonicalization will
/// be added without changing the transport boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPayload(String);

impl TextPayload {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, LogiscoreError> {
        String::from_utf8(bytes).map(Self).map_err(Into::into)
    }
}

const SOURCE_FILE_SCHEMA_VERSION: u8 = 1;
const UTF8_ENCODING: u8 = 0;
const SOURCE_FILE_HEADER_LENGTH: usize = 9;
const MAX_FILENAME_LENGTH: usize = 255;
pub(crate) const MAX_EXTENSION_LENGTH: usize = 32;
pub(crate) const MAX_SOURCE_LENGTH: usize = 8 * 1024 * 1024;

/// Canonical v2 representation of one UTF-8 source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFilePayload {
    filename: String,
    extension: String,
    source: String,
}

impl SourceFilePayload {
    pub fn new(
        filename: impl Into<String>,
        extension: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<Self, LogiscoreError> {
        let payload = Self {
            filename: filename.into(),
            extension: extension.into(),
            source: source.into(),
        };
        payload.validate()?;
        Ok(payload)
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let filename = self.filename.into_bytes();
        let extension = self.extension.into_bytes();
        let source = self.source.into_bytes();
        let mut bytes = Vec::with_capacity(
            SOURCE_FILE_HEADER_LENGTH + filename.len() + extension.len() + source.len(),
        );
        bytes.push(SOURCE_FILE_SCHEMA_VERSION);
        bytes.push(UTF8_ENCODING);
        bytes.extend_from_slice(&(filename.len() as u16).to_be_bytes());
        bytes.push(extension.len() as u8);
        bytes.extend_from_slice(&(source.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&filename);
        bytes.extend_from_slice(&extension);
        bytes.extend_from_slice(&source);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LogiscoreError> {
        if bytes.len() < SOURCE_FILE_HEADER_LENGTH {
            return Err(invalid_source_file("header is truncated"));
        }
        if bytes[0] != SOURCE_FILE_SCHEMA_VERSION {
            return Err(invalid_source_file("unsupported schema version"));
        }
        if bytes[1] != UTF8_ENCODING {
            return Err(invalid_source_file("unsupported text encoding"));
        }

        let filename_length = usize::from(u16::from_be_bytes([bytes[2], bytes[3]]));
        let extension_length = usize::from(bytes[4]);
        let source_length = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
        let expected_length = SOURCE_FILE_HEADER_LENGTH
            .checked_add(filename_length)
            .and_then(|length| length.checked_add(extension_length))
            .and_then(|length| length.checked_add(source_length))
            .ok_or_else(|| invalid_source_file("length overflow"))?;
        if bytes.len() != expected_length {
            return Err(invalid_source_file("length mismatch"));
        }

        let filename_end = SOURCE_FILE_HEADER_LENGTH + filename_length;
        let extension_end = filename_end + extension_length;
        let filename = String::from_utf8(bytes[SOURCE_FILE_HEADER_LENGTH..filename_end].to_vec())?;
        let extension = String::from_utf8(bytes[filename_end..extension_end].to_vec())?;
        let source = String::from_utf8(bytes[extension_end..].to_vec())?;
        Self::new(filename, extension, source)
    }

    fn validate(&self) -> Result<(), LogiscoreError> {
        let invalid_filename = self.filename.is_empty()
            || self.filename.len() > MAX_FILENAME_LENGTH
            || matches!(self.filename.as_str(), "." | "..")
            || self.filename.contains(['/', '\\', '\0']);
        if invalid_filename {
            return Err(invalid_source_file("invalid filename"));
        }
        if !is_valid_extension(&self.extension) {
            return Err(invalid_source_file("invalid extension"));
        }
        if self.source.len() > MAX_SOURCE_LENGTH {
            return Err(invalid_source_file("source exceeds 8 MiB"));
        }
        Ok(())
    }
}

pub(crate) fn is_valid_extension(extension: &str) -> bool {
    extension.len() <= MAX_EXTENSION_LENGTH
        && extension.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | '+')
        })
}

fn invalid_source_file(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidPacket(format!("invalid source file payload: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_type_accepts_defined_values() {
        assert_eq!(PayloadType::try_from(0).unwrap(), PayloadType::Text);
        assert_eq!(PayloadType::try_from(1).unwrap(), PayloadType::SourceFile);
        assert_eq!(PayloadType::try_from(2).unwrap(), PayloadType::Project);
    }

    #[test]
    fn payload_type_rejects_reserved_value() {
        assert!(PayloadType::try_from(3).is_err());
    }

    #[test]
    fn text_payload_preserves_utf8() {
        let original = TextPayload::new("明日13時集合 🎵");
        let restored = TextPayload::from_bytes(original.clone().into_bytes()).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn source_file_payload_roundtrips_utf8() {
        let original = SourceFilePayload::new("挨拶", ".rs", "// こんにちは 🎵").unwrap();
        let restored = SourceFilePayload::from_bytes(&original.clone().into_bytes()).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn source_file_payload_rejects_unsafe_metadata() {
        assert!(SourceFilePayload::new("../secret", ".rs", "source").is_err());
        assert!(SourceFilePayload::new("main", "bad extension", "source").is_err());
        assert!(SourceFilePayload::new("", ".rs", "source").is_err());
        assert!(SourceFilePayload::new("Dockerfile", "Dockerfile", "source").is_ok());
    }

    #[test]
    fn source_file_payload_rejects_truncated_and_trailing_data() {
        assert!(SourceFilePayload::from_bytes(&[0; SOURCE_FILE_HEADER_LENGTH - 1]).is_err());
        let mut encoded = SourceFilePayload::new("main", ".rs", "source")
            .unwrap()
            .into_bytes();
        encoded.push(0);
        assert!(SourceFilePayload::from_bytes(&encoded).is_err());
    }

    #[test]
    fn source_file_payload_rejects_unknown_schema_and_encoding() {
        let encoded = SourceFilePayload::new("main", ".rs", "source")
            .unwrap()
            .into_bytes();
        let mut unknown_schema = encoded.clone();
        unknown_schema[0] += 1;
        assert!(SourceFilePayload::from_bytes(&unknown_schema).is_err());
        let mut unknown_encoding = encoded;
        unknown_encoding[1] += 1;
        assert!(SourceFilePayload::from_bytes(&unknown_encoding).is_err());
    }
}
