use crate::error::LogiscoreError;
use crate::error_correction::crc32_checksum;

const MAGIC: [u8; 4] = *b"LSCP";
const VERSION: u8 = 1;
const HEADER_BYTES: usize = 19;
const CHECKSUM_BYTES: usize = 4;
const MAX_TRANSFER_BYTES: usize = 0x00ff_ffff + 7;

pub const DEFAULT_CHECKPOINT_PAYLOAD_BYTES: usize = 512;
pub const MAX_CHECKPOINT_PAYLOAD_BYTES: usize = 768;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointChunk {
    transfer_id: u32,
    index: u16,
    count: u16,
    total_length: u32,
    payload: Vec<u8>,
}

impl CheckpointChunk {
    pub const fn transfer_id(&self) -> u32 {
        self.transfer_id
    }

    pub const fn index(&self) -> u16 {
        self.index
    }

    pub const fn count(&self) -> u16 {
        self.count
    }

    pub const fn total_length(&self) -> u32 {
        self.total_length
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn encode(&self) -> Result<Vec<u8>, LogiscoreError> {
        validate_fields(self)?;
        let mut bytes = Vec::with_capacity(HEADER_BYTES + self.payload.len() + CHECKSUM_BYTES);
        bytes.extend_from_slice(&MAGIC);
        bytes.push(VERSION);
        bytes.extend_from_slice(&self.transfer_id.to_be_bytes());
        bytes.extend_from_slice(&self.index.to_be_bytes());
        bytes.extend_from_slice(&self.count.to_be_bytes());
        bytes.extend_from_slice(&self.total_length.to_be_bytes());
        let payload_length = u16::try_from(self.payload.len())
            .map_err(|_| invalid_checkpoint("chunk payload length overflow"))?;
        bytes.extend_from_slice(&payload_length.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(&crc32_checksum(&bytes).to_be_bytes());
        Ok(bytes)
    }

    pub(super) fn new(
        transfer_id: u32,
        index: u16,
        count: u16,
        total_length: u32,
        payload: Vec<u8>,
    ) -> Result<Self, LogiscoreError> {
        let chunk = Self {
            transfer_id,
            index,
            count,
            total_length,
            payload,
        };
        validate_fields(&chunk)?;
        Ok(chunk)
    }
}

pub fn split_checkpoint_packet(
    packet: &[u8],
    chunk_payload_bytes: usize,
) -> Result<Vec<CheckpointChunk>, LogiscoreError> {
    if chunk_payload_bytes == 0 || chunk_payload_bytes > MAX_CHECKPOINT_PAYLOAD_BYTES {
        return Err(invalid_checkpoint(
            "chunk payload size must be between 1 and 768 bytes",
        ));
    }
    if packet.len() > MAX_TRANSFER_BYTES {
        return Err(invalid_checkpoint("checkpoint packet exceeds size limit"));
    }
    let chunk_count = packet.len().max(1).div_ceil(chunk_payload_bytes);
    let count = u16::try_from(chunk_count)
        .map_err(|_| invalid_checkpoint("checkpoint chunk count exceeds limit"))?;
    let total_length = u32::try_from(packet.len())
        .map_err(|_| invalid_checkpoint("checkpoint packet length overflow"))?;
    let transfer_id = crc32_checksum(packet);
    if packet.is_empty() {
        return Ok(vec![CheckpointChunk::new(
            transfer_id,
            0,
            1,
            0,
            Vec::new(),
        )?]);
    }
    packet
        .chunks(chunk_payload_bytes)
        .enumerate()
        .map(|(index, payload)| {
            CheckpointChunk::new(
                transfer_id,
                u16::try_from(index)
                    .map_err(|_| invalid_checkpoint("checkpoint index overflow"))?,
                count,
                total_length,
                payload.to_vec(),
            )
        })
        .collect()
}

pub fn decode_checkpoint_chunk(bytes: &[u8]) -> Result<CheckpointChunk, LogiscoreError> {
    if bytes.len() < HEADER_BYTES + CHECKSUM_BYTES {
        return Err(invalid_checkpoint("checkpoint chunk is truncated"));
    }
    if bytes.get(..4) != Some(MAGIC.as_slice()) {
        return Err(invalid_checkpoint("checkpoint magic does not match"));
    }
    if bytes.get(4) != Some(&VERSION) {
        return Err(invalid_checkpoint("checkpoint version is unsupported"));
    }
    let transfer_id = read_u32(bytes, 5, "transfer ID")?;
    let index = read_u16(bytes, 9, "chunk index")?;
    let count = read_u16(bytes, 11, "chunk count")?;
    let total_length = read_u32(bytes, 13, "total length")?;
    let payload_length = usize::from(read_u16(bytes, 17, "chunk length")?);
    let expected_length = HEADER_BYTES
        .checked_add(payload_length)
        .and_then(|length| length.checked_add(CHECKSUM_BYTES))
        .ok_or_else(|| invalid_checkpoint("checkpoint length overflow"))?;
    if bytes.len() != expected_length {
        return Err(invalid_checkpoint("checkpoint chunk length does not match"));
    }
    let checksum_offset = bytes.len() - CHECKSUM_BYTES;
    let expected_checksum = read_u32(bytes, checksum_offset, "chunk checksum")?;
    if crc32_checksum(&bytes[..checksum_offset]) != expected_checksum {
        return Err(invalid_checkpoint("checkpoint chunk CRC-32 mismatch"));
    }
    CheckpointChunk::new(
        transfer_id,
        index,
        count,
        total_length,
        bytes[HEADER_BYTES..checksum_offset].to_vec(),
    )
}

fn validate_fields(chunk: &CheckpointChunk) -> Result<(), LogiscoreError> {
    if chunk.count == 0 || chunk.index >= chunk.count {
        return Err(invalid_checkpoint("checkpoint chunk index is invalid"));
    }
    if chunk.payload.len() > MAX_CHECKPOINT_PAYLOAD_BYTES {
        return Err(invalid_checkpoint("checkpoint chunk payload exceeds limit"));
    }
    let total_length = usize::try_from(chunk.total_length)
        .map_err(|_| invalid_checkpoint("checkpoint total length overflow"))?;
    if total_length > MAX_TRANSFER_BYTES {
        return Err(invalid_checkpoint("checkpoint total length exceeds limit"));
    }
    if chunk.total_length == 0
        && (chunk.count != 1 || chunk.index != 0 || !chunk.payload.is_empty())
    {
        return Err(invalid_checkpoint("empty checkpoint metadata is invalid"));
    }
    if chunk.total_length > 0 && chunk.payload.is_empty() {
        return Err(invalid_checkpoint(
            "non-empty checkpoint has an empty chunk",
        ));
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize, name: &str) -> Result<u16, LogiscoreError> {
    let value = bytes
        .get(offset..offset + 2)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| invalid_checkpoint(&format!("checkpoint {name} is truncated")))?;
    Ok(u16::from_be_bytes(value))
}

fn read_u32(bytes: &[u8], offset: usize, name: &str) -> Result<u32, LogiscoreError> {
    let value = bytes
        .get(offset..offset + 4)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| invalid_checkpoint(&format!("checkpoint {name} is truncated")))?;
    Ok(u32::from_be_bytes(value))
}

fn invalid_checkpoint(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidCheckpoint(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_chunk_wire_roundtrip_boundaries() {
        for payload in [Vec::new(), vec![7], vec![9; 512], vec![11; 513]] {
            let chunks = split_checkpoint_packet(&payload, 512).unwrap();
            assert_eq!(chunks.len(), payload.len().max(1).div_ceil(512));
            for chunk in chunks {
                assert_eq!(
                    decode_checkpoint_chunk(&chunk.encode().unwrap()).unwrap(),
                    chunk
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_size_tampering_and_truncation() {
        assert!(split_checkpoint_packet(b"payload", 0).is_err());
        assert!(split_checkpoint_packet(b"payload", 769).is_err());
        let chunk = split_checkpoint_packet(b"payload", 512).unwrap().remove(0);
        let mut encoded = chunk.encode().unwrap();
        encoded[19] ^= 1;
        assert!(decode_checkpoint_chunk(&encoded).is_err());
        assert!(decode_checkpoint_chunk(&encoded[..20]).is_err());
    }
}
