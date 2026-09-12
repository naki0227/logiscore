use std::collections::BTreeMap;

use super::{
    decode_checkpoint_chunk, reconstruct_checkpoint_packet, split_checkpoint_packet,
    CheckpointChunk,
};
use crate::adaptive::{AcousticProfile, AcousticProfileId};
use crate::audio::{FixedPcmCodec, PcmProfile};
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, TextPayload};
use crate::v2::adaptive_audio::build_profile_packet;
use crate::v2_packet::{decode_packet_bytes_for_fec_and_flags, FIXED_AUDIO_CODEC_PROFILE};

const MAX_LOOPS: usize = 16;
const MAX_LOOP_SAMPLES: usize = 16_000_000;

pub fn encode_checkpoint_loop(
    packet: &[u8],
    chunk_payload_bytes: usize,
    loops: usize,
) -> Result<Vec<f32>, LogiscoreError> {
    if loops == 0 || loops > MAX_LOOPS {
        return Err(invalid_checkpoint(
            "checkpoint loop count must be between 1 and 16",
        ));
    }
    let profile = AcousticProfile::for_id(AcousticProfileId::FixedFallback);
    let pcm_profile = PcmProfile::with_timing_percent(profile.timing_percent)?;
    let codec = FixedPcmCodec::new(pcm_profile);
    let chunks = split_checkpoint_packet(packet, chunk_payload_bytes)?;
    let mut encoded_chunks = Vec::with_capacity(chunks.len());
    let mut loop_samples = 0usize;
    for chunk in chunks {
        let samples = codec.encode(&chunk.encode()?)?;
        loop_samples = loop_samples
            .checked_add(samples.len())
            .ok_or_else(|| invalid_checkpoint("checkpoint loop length overflow"))?;
        encoded_chunks.push(samples);
    }
    let total_samples = loop_samples
        .checked_mul(loops)
        .filter(|length| *length <= MAX_LOOP_SAMPLES)
        .ok_or_else(|| invalid_checkpoint("checkpoint loop recording exceeds sample limit"))?;
    let mut output = Vec::with_capacity(total_samples);
    for _ in 0..loops {
        for chunk in &encoded_chunks {
            output.extend_from_slice(chunk);
        }
    }
    Ok(output)
}

pub fn decode_checkpoint_loop_recording(
    samples: &[f32],
    sample_rate: u32,
) -> Result<Vec<u8>, LogiscoreError> {
    let profile = AcousticProfile::for_id(AcousticProfileId::FixedFallback);
    let pcm_profile = PcmProfile::with_timing_percent(profile.timing_percent)?;
    let packets =
        FixedPcmCodec::new(pcm_profile).decode_all_at_sample_rate(samples, sample_rate)?;
    let mut transfers = BTreeMap::<(u32, u16, u32), Vec<CheckpointChunk>>::new();
    for packet in packets {
        if let Ok(chunk) = decode_checkpoint_chunk(&packet) {
            transfers
                .entry((chunk.transfer_id(), chunk.count(), chunk.total_length()))
                .or_default()
                .push(chunk);
        }
    }
    let mut recovered = transfers
        .values()
        .filter_map(|observations| reconstruct_checkpoint_packet(observations).ok());
    let packet = recovered
        .next()
        .ok_or_else(|| invalid_checkpoint("no complete checkpoint transfer was found"))?;
    if recovered.next().is_some() {
        return Err(invalid_checkpoint(
            "multiple complete checkpoint transfers were found",
        ));
    }
    Ok(packet)
}

pub fn encode_text_checkpoint_loop(
    text: &str,
    chunk_payload_bytes: usize,
    loops: usize,
) -> Result<Vec<f32>, LogiscoreError> {
    let profile = AcousticProfile::for_id(AcousticProfileId::FixedFallback);
    let packet = build_profile_packet(PayloadType::Text, text.as_bytes(), profile)?;
    encode_checkpoint_loop(&packet, chunk_payload_bytes, loops)
}

pub fn decode_text_checkpoint_loop_recording(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, LogiscoreError> {
    let packet = decode_checkpoint_loop_recording(samples, sample_rate)?;
    let profile = AcousticProfile::for_id(AcousticProfileId::FixedFallback);
    let payload = decode_packet_bytes_for_fec_and_flags(
        &packet,
        PayloadType::Text,
        FIXED_AUDIO_CODEC_PROFILE,
        profile.fec_profile,
        profile.id as u8,
    )?;
    TextPayload::from_bytes(payload).map(|text| text.as_str().to_owned())
}

fn invalid_checkpoint(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidCheckpoint(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotated_two_loop_recording_reconstructs_original_packet() {
        let packet = b"checkpoint loop roundtrip across arbitrary recording start";
        let mut recording = encode_checkpoint_loop(packet, 16, 2).unwrap();
        let recording_offset = recording.len() / 5;
        recording.rotate_left(recording_offset);
        assert_eq!(
            decode_checkpoint_loop_recording(&recording, 8_000).unwrap(),
            packet
        );
    }

    #[test]
    fn incomplete_single_loop_recording_is_rejected() {
        let packet = vec![42; 40];
        let recording = encode_checkpoint_loop(&packet, 16, 1).unwrap();
        assert!(
            decode_checkpoint_loop_recording(&recording[..recording.len() / 2], 8_000).is_err()
        );
    }

    #[test]
    fn rejects_unsafe_loop_counts() {
        assert!(encode_checkpoint_loop(b"packet", 16, 0).is_err());
        assert!(encode_checkpoint_loop(b"packet", 16, MAX_LOOPS + 1).is_err());
    }

    #[test]
    fn text_roundtrips_when_recording_starts_inside_the_first_loop() {
        let text = "任意位置から録音 🎼";
        let mut recording = encode_text_checkpoint_loop(text, 512, 2).unwrap();
        let recording_offset = recording.len() / 4;
        recording.rotate_left(recording_offset);
        assert_eq!(
            decode_text_checkpoint_loop_recording(&recording, 8_000).unwrap(),
            text
        );
    }
}
