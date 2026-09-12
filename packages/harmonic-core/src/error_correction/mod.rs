mod crc32;
mod hamming;
mod interleave;
mod repetition;

pub(crate) use crc32::checksum as crc32_checksum;

use crate::error::LogiscoreError;

const CHECKSUM_BYTES: usize = 4;
const CODEWORD_BITS: usize = 13;
const INTERLEAVE_DEPTH: usize = 8;
const REPETITIONS: usize = 3;

#[derive(Debug, Clone, Copy)]
struct FecConfig {
    interleave_depth: usize,
    repetitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recovery {
    data: Vec<u8>,
    corrected_codewords: usize,
}

impl Recovery {
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    pub const fn corrected_codewords(&self) -> usize {
        self.corrected_codewords
    }
}

pub fn protect(data: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
    protect_with_config(
        data,
        FecConfig {
            interleave_depth: INTERLEAVE_DEPTH,
            repetitions: REPETITIONS,
        },
    )
}

pub(crate) fn protect_for_profile(data: &[u8], profile: u8) -> Result<Vec<u8>, LogiscoreError> {
    protect_with_config(data, config_for_profile(profile)?)
}

fn protect_with_config(data: &[u8], config: FecConfig) -> Result<Vec<u8>, LogiscoreError> {
    let protected_length = protected_length_with_config(data.len(), config)?;
    let mut framed = Vec::with_capacity(data.len() + CHECKSUM_BYTES);
    framed.extend_from_slice(data);
    framed.extend_from_slice(&crc32::checksum(data).to_be_bytes());

    let codeword_bits = hamming::encode(&framed);
    let interleaved = interleave::encode(&codeword_bits, CODEWORD_BITS, config.interleave_depth)?;
    let packed = pack_bits(&interleaved);
    let protected = repetition::encode(&packed, config.repetitions);
    debug_assert_eq!(protected.len(), protected_length);
    Ok(protected)
}

pub fn protected_length(data_length: usize) -> Result<usize, LogiscoreError> {
    protected_length_with_config(
        data_length,
        FecConfig {
            interleave_depth: INTERLEAVE_DEPTH,
            repetitions: REPETITIONS,
        },
    )
}

pub(crate) fn protected_length_for_profile(
    data_length: usize,
    profile: u8,
) -> Result<usize, LogiscoreError> {
    protected_length_with_config(data_length, config_for_profile(profile)?)
}

fn protected_length_with_config(
    data_length: usize,
    config: FecConfig,
) -> Result<usize, LogiscoreError> {
    data_length
        .checked_add(CHECKSUM_BYTES)
        .and_then(|length| length.checked_mul(CODEWORD_BITS))
        .and_then(|bits| bits.checked_add(7))
        .map(|bits| bits / 8)
        .and_then(|packed| packed.checked_mul(config.repetitions))
        .ok_or_else(|| invalid_fec("protected length overflow"))
}

pub fn recover(protected: &[u8]) -> Result<Recovery, LogiscoreError> {
    recover_with_config(
        protected,
        FecConfig {
            interleave_depth: INTERLEAVE_DEPTH,
            repetitions: REPETITIONS,
        },
    )
}

pub(crate) fn recover_for_profile(
    protected: &[u8],
    profile: u8,
) -> Result<Recovery, LogiscoreError> {
    recover_with_config(protected, config_for_profile(profile)?)
}

fn recover_with_config(protected: &[u8], config: FecConfig) -> Result<Recovery, LogiscoreError> {
    let voted = repetition::decode(protected, config.repetitions)?;
    let framed_bytes = infer_framed_length(voted.len())?;
    let encoded_bit_length = framed_bytes
        .checked_mul(CODEWORD_BITS)
        .ok_or_else(|| invalid_fec("encoded bit length overflow"))?;
    let packed_bits = unpack_bits(&voted, encoded_bit_length)?;
    let codeword_bits = interleave::decode(
        &packed_bits,
        CODEWORD_BITS,
        config.interleave_depth,
        framed_bytes,
    )?;
    let decoded = hamming::decode(&codeword_bits)?;
    if decoded.data.len() < CHECKSUM_BYTES {
        return Err(invalid_fec("checksum is missing"));
    }
    let data_length = decoded.data.len() - CHECKSUM_BYTES;
    let expected_checksum = u32::from_be_bytes(
        decoded.data[data_length..]
            .try_into()
            .map_err(|_| invalid_fec("checksum is malformed"))?,
    );
    let data = decoded.data[..data_length].to_vec();
    if crc32::checksum(&data) != expected_checksum {
        return Err(invalid_fec("CRC-32 mismatch"));
    }
    Ok(Recovery {
        data,
        corrected_codewords: decoded.corrected_codewords,
    })
}

fn config_for_profile(profile: u8) -> Result<FecConfig, LogiscoreError> {
    match profile {
        1 => Ok(FecConfig {
            interleave_depth: 8,
            repetitions: 3,
        }),
        2 => Ok(FecConfig {
            interleave_depth: 16,
            repetitions: 3,
        }),
        3 => Ok(FecConfig {
            interleave_depth: 16,
            repetitions: 5,
        }),
        _ => Err(invalid_fec("unsupported protected FEC profile")),
    }
}

fn infer_framed_length(packed_length: usize) -> Result<usize, LogiscoreError> {
    let bit_length = packed_length
        .checked_mul(8)
        .ok_or_else(|| invalid_fec("protected length overflow"))?;
    let framed_length = bit_length / CODEWORD_BITS;
    let expected_packed_length = framed_length
        .checked_mul(CODEWORD_BITS)
        .and_then(|bits| bits.checked_add(7))
        .map(|bits| bits / 8)
        .ok_or_else(|| invalid_fec("protected length overflow"))?;
    if framed_length < CHECKSUM_BYTES || expected_packed_length != packed_length {
        return Err(invalid_fec("protected length is invalid"));
    }
    Ok(framed_length)
}

fn pack_bits(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0, |byte, bit| (byte << 1) | u8::from(*bit))
                << (8 - chunk.len())
        })
        .collect()
}

fn unpack_bits(bytes: &[u8], bit_length: usize) -> Result<Vec<bool>, LogiscoreError> {
    if bit_length > bytes.len().saturating_mul(8) {
        return Err(invalid_fec("protected data is truncated"));
    }
    Ok((0..bit_length)
        .map(|offset| (bytes[offset / 8] >> (7 - offset % 8)) & 1 == 1)
        .collect())
}

fn invalid_fec(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidFec(message.to_owned())
}

#[cfg(test)]
mod tests;
