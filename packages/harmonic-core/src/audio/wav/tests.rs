use super::*;

#[test]
fn pcm16_wav_roundtrips_with_expected_quantization() {
    let audio = PcmAudio::new(8_000, vec![-1.0, -0.5, 0.0, 0.5, 1.0]).unwrap();
    let decoded = decode_wav(&encode_wav(&audio).unwrap()).unwrap();
    assert_eq!(decoded.sample_rate(), 8_000);
    for (actual, expected) in decoded.samples().iter().zip(audio.samples()) {
        assert!((actual - expected).abs() <= 1.0 / f32::from(i16::MAX));
    }
}

#[test]
fn decoder_skips_unknown_even_sized_chunk() {
    let audio = PcmAudio::new(8_000, vec![0.25]).unwrap();
    let mut wav = encode_wav(&audio).unwrap();
    wav.splice(12..12, *b"JUNK\x02\x00\x00\x00ok");
    let riff_length = (wav.len() - 8) as u32;
    wav[4..8].copy_from_slice(&riff_length.to_le_bytes());
    assert_eq!(decode_wav(&wav).unwrap().samples().len(), 1);
}

#[test]
fn decoder_skips_unknown_odd_sized_chunk_and_padding() {
    let audio = PcmAudio::new(8_000, vec![0.25]).unwrap();
    let mut wav = encode_wav(&audio).unwrap();
    wav.splice(12..12, *b"JUNK\x01\x00\x00\x00x\x00");
    let riff_length = (wav.len() - 8) as u32;
    wav[4..8].copy_from_slice(&riff_length.to_le_bytes());
    assert_eq!(decode_wav(&wav).unwrap().samples().len(), 1);
}

#[test]
fn decoder_downmixes_stereo_pcm16() {
    let mut data = Vec::new();
    for sample in [i16::MAX, -i16::MAX, i16::MAX / 2, i16::MAX / 2] {
        data.extend_from_slice(&sample.to_le_bytes());
    }
    let decoded = decode_wav(&build_wav(PCM_FORMAT, 2, 16, &data)).unwrap();
    assert_eq!(decoded.samples().len(), 2);
    assert!(decoded.samples()[0].abs() < 0.0001);
    assert!((decoded.samples()[1] - 0.5).abs() < 0.0001);
}

#[test]
fn decoder_accepts_mono_float32() {
    let data = [0.25f32, -0.5]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let decoded = decode_wav(&build_wav(FLOAT_FORMAT, 1, 32, &data)).unwrap();
    assert_eq!(decoded.samples(), &[0.25, -0.5]);
}

#[test]
fn decoder_rejects_bad_lengths_and_unsupported_encoding() {
    let audio = PcmAudio::new(8_000, vec![0.0]).unwrap();
    let mut truncated = encode_wav(&audio).unwrap();
    truncated.pop();
    assert!(decode_wav(&truncated).is_err());

    let mut unsupported = encode_wav(&audio).unwrap();
    unsupported[20..22].copy_from_slice(&6u16.to_le_bytes());
    assert!(decode_wav(&unsupported).is_err());
}

fn build_wav(encoding: u16, channels: u16, bits: u16, data: &[u8]) -> Vec<u8> {
    let block_align = channels * (bits / 8);
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&encoding.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&8_000u32.to_le_bytes());
    wav.extend_from_slice(&(8_000u32 * u32::from(block_align)).to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
    wav.extend_from_slice(data);
    wav
}
