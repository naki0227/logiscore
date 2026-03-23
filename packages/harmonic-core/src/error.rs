use thiserror::Error;

/// Logiscore で発生し得るエラーの列挙型。
#[derive(Error, Debug)]
pub enum LogiscoreError {
    #[error("Compression/decompression failed: {0}")]
    CompressionError(#[from] std::io::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeader(u8),

    #[error("MIDI note {0} not found in scale")]
    NoteNotInScale(u8),

    #[error("Decoded data is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("MIDI parse error: {0}")]
    MidiParseError(String),

    #[error("Invalid MIDI: {0}")]
    InvalidMidi(String),

    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(String),
}
