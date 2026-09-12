use std::collections::BTreeMap;

use super::frame::CheckpointChunk;
use crate::error::LogiscoreError;
use crate::error_correction::crc32_checksum;

pub fn reconstruct_checkpoint_packet(
    observations: &[CheckpointChunk],
) -> Result<Vec<u8>, LogiscoreError> {
    let first = observations
        .first()
        .ok_or_else(|| invalid_checkpoint("checkpoint observations are empty"))?;
    let identity = (first.transfer_id(), first.count(), first.total_length());
    let mut variants = BTreeMap::<u16, BTreeMap<Vec<u8>, usize>>::new();
    for observation in observations {
        if (
            observation.transfer_id(),
            observation.count(),
            observation.total_length(),
        ) != identity
        {
            return Err(invalid_checkpoint(
                "checkpoint observations mix different transfers",
            ));
        }
        *variants
            .entry(observation.index())
            .or_default()
            .entry(observation.payload().to_vec())
            .or_default() += 1;
    }
    if variants.len() != usize::from(first.count()) {
        return Err(invalid_checkpoint("checkpoint observations are incomplete"));
    }
    let mut packet = Vec::with_capacity(first.total_length() as usize);
    for index in 0..first.count() {
        let candidates = variants
            .get(&index)
            .ok_or_else(|| invalid_checkpoint("checkpoint chunk is missing"))?;
        let max_votes = candidates
            .values()
            .copied()
            .max()
            .ok_or_else(|| invalid_checkpoint("checkpoint chunk has no observation"))?;
        let mut winners = candidates
            .iter()
            .filter(|(_, votes)| **votes == max_votes)
            .map(|(payload, _)| payload);
        let winner = winners
            .next()
            .ok_or_else(|| invalid_checkpoint("checkpoint chunk has no observation"))?;
        if winners.next().is_some() {
            return Err(invalid_checkpoint("checkpoint chunk vote is tied"));
        }
        packet.extend_from_slice(winner);
        if packet.len() > first.total_length() as usize {
            return Err(invalid_checkpoint(
                "checkpoint payload exceeds declared length",
            ));
        }
    }
    if packet.len() != first.total_length() as usize {
        return Err(invalid_checkpoint(
            "checkpoint payload length does not match",
        ));
    }
    if crc32_checksum(&packet) != first.transfer_id() {
        return Err(invalid_checkpoint("checkpoint transfer CRC-32 mismatch"));
    }
    Ok(packet)
}

fn invalid_checkpoint(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidCheckpoint(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::checkpoint::split_checkpoint_packet;

    #[test]
    fn reconstructs_rotated_loop_with_duplicate_observations() {
        let packet = (0..=255).cycle().take(1_300).collect::<Vec<_>>();
        let chunks = split_checkpoint_packet(&packet, 512).unwrap();
        let observations = vec![
            chunks[1].clone(),
            chunks[2].clone(),
            chunks[0].clone(),
            chunks[1].clone(),
        ];
        assert_eq!(
            reconstruct_checkpoint_packet(&observations).unwrap(),
            packet
        );
    }

    #[test]
    fn majority_selects_the_repeated_valid_chunk() {
        let packet = b"majority".to_vec();
        let correct = split_checkpoint_packet(&packet, 512).unwrap().remove(0);
        let alternate = CheckpointChunk::new(
            correct.transfer_id(),
            0,
            1,
            correct.total_length(),
            b"minority".to_vec(),
        )
        .unwrap();
        assert_eq!(
            reconstruct_checkpoint_packet(&[correct.clone(), alternate, correct]).unwrap(),
            packet
        );
    }

    #[test]
    fn rejects_missing_mixed_tied_and_wrong_full_crc() {
        let packet = vec![1; 600];
        let chunks = split_checkpoint_packet(&packet, 512).unwrap();
        assert!(reconstruct_checkpoint_packet(&chunks[..1]).is_err());

        let other = split_checkpoint_packet(b"other transfer", 512)
            .unwrap()
            .remove(0);
        assert!(reconstruct_checkpoint_packet(&[chunks[0].clone(), other]).is_err());

        let tied = CheckpointChunk::new(
            chunks[0].transfer_id(),
            0,
            chunks[0].count(),
            chunks[0].total_length(),
            vec![2; 512],
        )
        .unwrap();
        assert!(
            reconstruct_checkpoint_packet(&[chunks[0].clone(), tied, chunks[1].clone()]).is_err()
        );

        let wrong_crc = CheckpointChunk::new(7, 0, 1, 3, b"bad".to_vec()).unwrap();
        assert!(reconstruct_checkpoint_packet(&[wrong_crc]).is_err());
    }
}
