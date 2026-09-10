use crate::error::LogiscoreError;

const MAX_WAV_BYTES: usize = 64 * 1024 * 1024;
const MAX_SAMPLE_RATE: u32 = 384_000;
const MAX_CHANNELS: u16 = 32;
const PCM_FORMAT: u16 = 1;
const FLOAT_FORMAT: u16 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct PcmAudio {
    sample_rate: u32,
    samples: Vec<f32>,
}

impl PcmAudio {
    pub fn new(sample_rate: u32, samples: Vec<f32>) -> Result<Self, LogiscoreError> {
        if sample_rate == 0 || sample_rate > MAX_SAMPLE_RATE {
            return Err(invalid_audio("sample rate is outside the supported range"));
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(invalid_audio("PCM contains non-finite samples"));
        }
        Ok(Self {
            sample_rate,
            samples,
        })
    }

    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn into_samples(self) -> Vec<f32> {
        self.samples
    }
}

pub fn encode_wav(audio: &PcmAudio) -> Result<Vec<u8>, LogiscoreError> {
    let data_length = audio
        .samples
        .len()
        .checked_mul(2)
        .ok_or_else(|| invalid_audio("WAV data length overflow"))?;
    let riff_length = 36usize
        .checked_add(data_length)
        .ok_or_else(|| invalid_audio("WAV RIFF length overflow"))?;
    if riff_length > u32::MAX as usize || riff_length + 8 > MAX_WAV_BYTES {
        return Err(invalid_audio("WAV exceeds the file size limit"));
    }
    let mut output = Vec::with_capacity(riff_length + 8);
    output.extend_from_slice(b"RIFF");
    output.extend_from_slice(&(riff_length as u32).to_le_bytes());
    output.extend_from_slice(b"WAVEfmt ");
    output.extend_from_slice(&16u32.to_le_bytes());
    output.extend_from_slice(&PCM_FORMAT.to_le_bytes());
    output.extend_from_slice(&1u16.to_le_bytes());
    output.extend_from_slice(&audio.sample_rate.to_le_bytes());
    output.extend_from_slice(&(audio.sample_rate * 2).to_le_bytes());
    output.extend_from_slice(&2u16.to_le_bytes());
    output.extend_from_slice(&16u16.to_le_bytes());
    output.extend_from_slice(b"data");
    output.extend_from_slice(&(data_length as u32).to_le_bytes());
    for &sample in &audio.samples {
        let quantized = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        output.extend_from_slice(&quantized.to_le_bytes());
    }
    Ok(output)
}

pub fn decode_wav(bytes: &[u8]) -> Result<PcmAudio, LogiscoreError> {
    if bytes.len() < 12 || bytes.len() > MAX_WAV_BYTES {
        return Err(invalid_audio("WAV size is invalid"));
    }
    if &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(invalid_audio("WAV RIFF header is invalid"));
    }
    let declared_length = read_u32(bytes, 4)? as usize + 8;
    if declared_length != bytes.len() {
        return Err(invalid_audio("WAV RIFF length mismatch"));
    }

    let mut format = None;
    let mut data = None;
    let mut cursor = 12usize;
    while cursor < bytes.len() {
        let header_end = cursor
            .checked_add(8)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid_audio("WAV chunk header is truncated"))?;
        let chunk_length = read_u32(bytes, cursor + 4)? as usize;
        let chunk_start = header_end;
        let chunk_end = chunk_start
            .checked_add(chunk_length)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid_audio("WAV chunk is truncated"))?;
        match &bytes[cursor..cursor + 4] {
            b"fmt " => format = Some(parse_format(&bytes[chunk_start..chunk_end])?),
            b"data" => data = Some(&bytes[chunk_start..chunk_end]),
            _ => {}
        }
        cursor = chunk_end
            .checked_add(chunk_length % 2)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid_audio("WAV chunk padding is truncated"))?;
    }
    let format = format.ok_or_else(|| invalid_audio("WAV fmt chunk is missing"))?;
    let data = data.ok_or_else(|| invalid_audio("WAV data chunk is missing"))?;
    decode_samples(format, data)
}

#[derive(Clone, Copy)]
struct Format {
    encoding: u16,
    channels: u16,
    sample_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn parse_format(bytes: &[u8]) -> Result<Format, LogiscoreError> {
    if bytes.len() < 16 {
        return Err(invalid_audio("WAV fmt chunk is truncated"));
    }
    let format = Format {
        encoding: read_u16(bytes, 0)?,
        channels: read_u16(bytes, 2)?,
        sample_rate: read_u32(bytes, 4)?,
        block_align: read_u16(bytes, 12)?,
        bits_per_sample: read_u16(bytes, 14)?,
    };
    if format.channels == 0
        || format.channels > MAX_CHANNELS
        || format.sample_rate == 0
        || format.sample_rate > MAX_SAMPLE_RATE
    {
        return Err(invalid_audio("WAV format dimensions are invalid"));
    }
    let supported = (format.encoding == PCM_FORMAT && format.bits_per_sample == 16)
        || (format.encoding == FLOAT_FORMAT && format.bits_per_sample == 32);
    let expected_align = format
        .channels
        .checked_mul(format.bits_per_sample / 8)
        .ok_or_else(|| invalid_audio("WAV block alignment overflow"))?;
    if !supported || format.block_align != expected_align {
        return Err(invalid_audio("WAV encoding is unsupported"));
    }
    Ok(format)
}

fn decode_samples(format: Format, data: &[u8]) -> Result<PcmAudio, LogiscoreError> {
    let frame_bytes = usize::from(format.block_align);
    if !data.len().is_multiple_of(frame_bytes) {
        return Err(invalid_audio("WAV sample frame is truncated"));
    }
    let sample_bytes = usize::from(format.bits_per_sample / 8);
    let mut mono = Vec::with_capacity(data.len() / frame_bytes);
    for frame in data.chunks_exact(frame_bytes) {
        let mut sum = 0.0;
        for channel in 0..usize::from(format.channels) {
            let offset = channel * sample_bytes;
            sum += if format.encoding == PCM_FORMAT {
                f32::from(i16::from_le_bytes([frame[offset], frame[offset + 1]]))
                    / f32::from(i16::MAX)
            } else {
                f32::from_le_bytes(
                    frame[offset..offset + 4]
                        .try_into()
                        .map_err(|_| invalid_audio("WAV floating-point sample is truncated"))?,
                )
            };
        }
        let sample = sum / f32::from(format.channels);
        if !sample.is_finite() {
            return Err(invalid_audio("WAV contains non-finite samples"));
        }
        mono.push(sample);
    }
    PcmAudio::new(format.sample_rate, mono)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, LogiscoreError> {
    bytes
        .get(offset..offset + 2)
        .and_then(|value| value.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| invalid_audio("WAV integer is truncated"))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, LogiscoreError> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| invalid_audio("WAV integer is truncated"))
}

fn invalid_audio(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidAudio(message.to_owned())
}

#[cfg(test)]
mod tests;
