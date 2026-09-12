use std::collections::BTreeMap;

use super::frame::{decode_checkpoint_chunk, decode_checkpoint_chunk_unchecked, CheckpointChunk};
use crate::audio::MiniPcmObservation;

type ObservationKey = (u32, u16, u16, u32, usize);

pub(super) fn recover_soft_chunks(observations: &[MiniPcmObservation]) -> Vec<CheckpointChunk> {
    let mut grouped = BTreeMap::<ObservationKey, Vec<&MiniPcmObservation>>::new();
    for observation in observations {
        if !valid_confidences(observation) {
            continue;
        }
        if let Ok(chunk) = decode_checkpoint_chunk_unchecked(&observation.bytes) {
            grouped
                .entry((
                    chunk.transfer_id(),
                    chunk.index(),
                    chunk.count(),
                    chunk.total_length(),
                    observation.bytes.len(),
                ))
                .or_default()
                .push(observation);
        }
    }

    grouped
        .values()
        .filter(|group| group.len() >= 2)
        .filter_map(|group| combine_observations(group))
        .filter_map(|bytes| decode_checkpoint_chunk(&bytes).ok())
        .collect()
}

fn valid_confidences(observation: &MiniPcmObservation) -> bool {
    observation.nibble_confidences.len() == observation.bytes.len().saturating_mul(2)
        && observation
            .nibble_confidences
            .iter()
            .all(|confidence| confidence.is_finite() && *confidence >= 0.0)
}

fn combine_observations(observations: &[&MiniPcmObservation]) -> Option<Vec<u8>> {
    let byte_length = observations.first()?.bytes.len();
    let mut combined = Vec::with_capacity(byte_length);
    for byte_index in 0..byte_length {
        let high = combine_nibble(observations, byte_index * 2)?;
        let low = combine_nibble(observations, byte_index * 2 + 1)?;
        combined.push((high << 4) | low);
    }
    Some(combined)
}

fn combine_nibble(observations: &[&MiniPcmObservation], nibble_index: usize) -> Option<u8> {
    let mut weights = [0.0f32; 16];
    for observation in observations {
        let byte = *observation.bytes.get(nibble_index / 2)?;
        let nibble = if nibble_index.is_multiple_of(2) {
            byte >> 4
        } else {
            byte & 0x0f
        };
        let confidence = *observation.nibble_confidences.get(nibble_index)?;
        weights[usize::from(nibble)] += confidence.max(0.001);
    }
    let mut ranked = weights.into_iter().enumerate().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    if ranked[0].1 <= ranked[1].1 {
        return None;
    }
    u8::try_from(ranked[0].0).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::checkpoint::split_checkpoint_packet;

    fn observation(bytes: Vec<u8>) -> MiniPcmObservation {
        MiniPcmObservation {
            nibble_confidences: vec![1.0; bytes.len() * 2],
            bytes,
        }
    }

    #[test]
    fn combines_differently_damaged_crc_invalid_observations() {
        let original = split_checkpoint_packet(b"soft combine", 512)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        let mut first = original.clone();
        let mut second = original.clone();
        let mut third = original.clone();
        first[19] ^= 0x10;
        second[20] ^= 0x01;
        third[21] ^= 0x10;
        assert!(decode_checkpoint_chunk(&first).is_err());
        assert!(decode_checkpoint_chunk(&second).is_err());
        assert!(decode_checkpoint_chunk(&third).is_err());

        let recovered =
            recover_soft_chunks(&[observation(first), observation(second), observation(third)]);
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].payload(), b"soft combine");
    }

    #[test]
    fn rejects_invalid_confidence_shape_and_tied_nibbles() {
        let original = split_checkpoint_packet(b"tie", 512)
            .unwrap()
            .remove(0)
            .encode()
            .unwrap();
        let mut alternate = original.clone();
        alternate[19] ^= 0x10;
        let invalid = MiniPcmObservation {
            bytes: original.clone(),
            nibble_confidences: vec![],
        };
        assert!(recover_soft_chunks(&[invalid]).is_empty());
        assert!(recover_soft_chunks(&[observation(original), observation(alternate)]).is_empty());
    }
}
