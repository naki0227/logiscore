use crate::error::LogiscoreError;
use crate::payload::{is_valid_extension, MAX_SOURCE_LENGTH};

const PROJECT_SCHEMA_VERSION: u8 = 1;
const UTF8_ENCODING: u8 = 0;
const PROJECT_HEADER_LENGTH: usize = 8;
const ENTRY_HEADER_LENGTH: usize = 7;
const MAX_FILES: usize = 2_048;
const MAX_PATH_LENGTH: usize = 1_024;
const MAX_PATH_SEGMENT_LENGTH: usize = 255;
const MAX_TOTAL_SOURCE_LENGTH: usize = 32 * 1024 * 1024;

/// One regular UTF-8 file in a canonical v2 project archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFilePayload {
    path: String,
    extension: String,
    source: String,
}

impl ProjectFilePayload {
    pub fn new(
        path: impl Into<String>,
        extension: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<Self, LogiscoreError> {
        let file = Self {
            path: path.into(),
            extension: extension.into(),
            source: source.into(),
        };
        if !is_valid_project_path(&file.path) {
            return Err(invalid_project("invalid file path"));
        }
        if !is_valid_extension(&file.extension) {
            return Err(invalid_project("invalid extension"));
        }
        if file.source.len() > MAX_SOURCE_LENGTH {
            return Err(invalid_project("file source exceeds 8 MiB"));
        }
        Ok(file)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Canonical archive containing regular files sorted by UTF-8 path bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPayload {
    files: Vec<ProjectFilePayload>,
}

impl ProjectPayload {
    pub fn new(mut files: Vec<ProjectFilePayload>) -> Result<Self, LogiscoreError> {
        if files.is_empty() {
            return Err(invalid_project("project must contain at least one file"));
        }
        if files.len() > MAX_FILES {
            return Err(invalid_project("project exceeds 2048 files"));
        }
        files.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
        if files.windows(2).any(|pair| pair[0].path == pair[1].path) {
            return Err(invalid_project("duplicate file path"));
        }
        checked_total_source_length(&files)?;
        Ok(Self { files })
    }

    pub fn files(&self) -> &[ProjectFilePayload] {
        &self.files
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let total_source_length = self
            .files
            .iter()
            .map(|file| file.source.len())
            .sum::<usize>();
        let capacity = PROJECT_HEADER_LENGTH
            + self
                .files
                .iter()
                .map(|file| {
                    ENTRY_HEADER_LENGTH + file.path.len() + file.extension.len() + file.source.len()
                })
                .sum::<usize>();
        let mut bytes = Vec::with_capacity(capacity);
        bytes.push(PROJECT_SCHEMA_VERSION);
        bytes.push(UTF8_ENCODING);
        bytes.extend_from_slice(&(self.files.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&(total_source_length as u32).to_be_bytes());
        for file in self.files {
            bytes.extend_from_slice(&(file.path.len() as u16).to_be_bytes());
            bytes.push(file.extension.len() as u8);
            bytes.extend_from_slice(&(file.source.len() as u32).to_be_bytes());
            bytes.extend_from_slice(file.path.as_bytes());
            bytes.extend_from_slice(file.extension.as_bytes());
            bytes.extend_from_slice(file.source.as_bytes());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LogiscoreError> {
        if bytes.len() < PROJECT_HEADER_LENGTH {
            return Err(invalid_project("header is truncated"));
        }
        if bytes[0] != PROJECT_SCHEMA_VERSION {
            return Err(invalid_project("unsupported schema version"));
        }
        if bytes[1] != UTF8_ENCODING {
            return Err(invalid_project("unsupported text encoding"));
        }
        let file_count = usize::from(u16::from_be_bytes([bytes[2], bytes[3]]));
        if !(1..=MAX_FILES).contains(&file_count) {
            return Err(invalid_project("invalid file count"));
        }
        let declared_source_length =
            u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        if declared_source_length > MAX_TOTAL_SOURCE_LENGTH {
            return Err(invalid_project("total source exceeds 32 MiB"));
        }

        let mut cursor = PROJECT_HEADER_LENGTH;
        let mut files = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let entry_header_end = cursor
                .checked_add(ENTRY_HEADER_LENGTH)
                .filter(|end| *end <= bytes.len())
                .ok_or_else(|| invalid_project("entry header is truncated"))?;
            let path_length = usize::from(u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]));
            let extension_length = usize::from(bytes[cursor + 2]);
            let source_length = u32::from_be_bytes([
                bytes[cursor + 3],
                bytes[cursor + 4],
                bytes[cursor + 5],
                bytes[cursor + 6],
            ]) as usize;
            cursor = entry_header_end;
            let path_end = checked_end(cursor, path_length, bytes.len())?;
            let extension_end = checked_end(path_end, extension_length, bytes.len())?;
            let source_end = checked_end(extension_end, source_length, bytes.len())?;
            let path = String::from_utf8(bytes[cursor..path_end].to_vec())?;
            let extension = String::from_utf8(bytes[path_end..extension_end].to_vec())?;
            let source = String::from_utf8(bytes[extension_end..source_end].to_vec())?;
            let file = ProjectFilePayload::new(path, extension, source)?;
            if files.last().is_some_and(|previous: &ProjectFilePayload| {
                previous.path.as_bytes() >= file.path.as_bytes()
            }) {
                return Err(invalid_project("entries are not in canonical path order"));
            }
            files.push(file);
            cursor = source_end;
        }
        if cursor != bytes.len() {
            return Err(invalid_project("trailing archive data"));
        }
        if checked_total_source_length(&files)? != declared_source_length {
            return Err(invalid_project("total source length mismatch"));
        }
        Ok(Self { files })
    }
}

fn checked_end(start: usize, length: usize, available: usize) -> Result<usize, LogiscoreError> {
    start
        .checked_add(length)
        .filter(|end| *end <= available)
        .ok_or_else(|| invalid_project("entry data is truncated"))
}

fn checked_total_source_length(files: &[ProjectFilePayload]) -> Result<usize, LogiscoreError> {
    let total = files.iter().try_fold(0usize, |total, file| {
        total
            .checked_add(file.source.len())
            .ok_or_else(|| invalid_project("total source length overflow"))
    })?;
    if total > MAX_TOTAL_SOURCE_LENGTH {
        return Err(invalid_project("total source exceeds 32 MiB"));
    }
    Ok(total)
}

fn is_valid_project_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= MAX_PATH_LENGTH
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains(['\\', '\0', ':'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment.len() <= MAX_PATH_SEGMENT_LENGTH
        })
}

fn invalid_project(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidPacket(format!("invalid project payload: {message}"))
}

#[cfg(test)]
mod tests;
