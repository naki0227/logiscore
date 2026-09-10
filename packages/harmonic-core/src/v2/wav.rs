use crate::audio::{decode_wav, encode_wav, PcmAudio, PcmCodec};
use crate::error::LogiscoreError;
use crate::payload::{PayloadType, SourceFilePayload, TextPayload};
use crate::project_payload::{ProjectFilePayload, ProjectPayload};
use crate::v2_packet::{decode_packet_bytes_for_fec, RELIABLE_FEC_PROFILE, RHYTHMIC_CODEC_PROFILE};

pub fn encode_text_wav_reliable(text: &str) -> Result<Vec<u8>, LogiscoreError> {
    pcm_to_wav(super::encode_text_pcm_reliable(text)?)
}

pub fn decode_text_wav_reliable(bytes: &[u8]) -> Result<String, LogiscoreError> {
    let payload = decode_wav_payload(bytes, PayloadType::Text)?;
    TextPayload::from_bytes(payload).map(|text| text.as_str().to_owned())
}

pub fn encode_source_file_wav_reliable(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, LogiscoreError> {
    pcm_to_wav(super::encode_source_file_pcm_reliable(
        filename, extension, source,
    )?)
}

pub fn decode_source_file_wav_reliable(bytes: &[u8]) -> Result<SourceFilePayload, LogiscoreError> {
    SourceFilePayload::from_bytes(&decode_wav_payload(bytes, PayloadType::SourceFile)?)
}

pub fn encode_project_wav_reliable(
    files: Vec<ProjectFilePayload>,
) -> Result<Vec<u8>, LogiscoreError> {
    pcm_to_wav(super::encode_project_pcm_reliable(files)?)
}

pub fn decode_project_wav_reliable(bytes: &[u8]) -> Result<ProjectPayload, LogiscoreError> {
    ProjectPayload::from_bytes(&decode_wav_payload(bytes, PayloadType::Project)?)
}

fn pcm_to_wav(samples: Vec<f32>) -> Result<Vec<u8>, LogiscoreError> {
    encode_wav(&PcmAudio::new(super::pcm_sample_rate(), samples)?)
}

fn decode_wav_payload(bytes: &[u8], payload_type: PayloadType) -> Result<Vec<u8>, LogiscoreError> {
    let audio = decode_wav(bytes)?;
    let packet = PcmCodec::default().decode_at_sample_rate(audio.samples(), audio.sample_rate())?;
    decode_packet_bytes_for_fec(
        &packet,
        payload_type,
        RHYTHMIC_CODEC_PROFILE,
        RELIABLE_FEC_PROFILE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reliable_wav_roundtrips_all_payload_types() {
        let text = encode_text_wav_reliable("WAVから復元 🎙️").unwrap();
        assert_eq!(decode_text_wav_reliable(&text).unwrap(), "WAVから復元 🎙️");

        let source = encode_source_file_wav_reliable("main.rs", ".rs", "fn main() {}").unwrap();
        assert_eq!(
            decode_source_file_wav_reliable(&source).unwrap().source(),
            "fn main() {}"
        );

        let files = vec![ProjectFilePayload::new("README.md", ".md", "# WAV").unwrap()];
        let project = encode_project_wav_reliable(files).unwrap();
        assert_eq!(
            decode_project_wav_reliable(&project).unwrap().files().len(),
            1
        );
    }

    #[test]
    fn decoder_resamples_a_48khz_recording() {
        let samples = super::super::encode_text_pcm_reliable("48 kHz").unwrap();
        let upsampled = samples
            .into_iter()
            .flat_map(|sample| std::iter::repeat_n(sample, 6))
            .collect();
        let wav = encode_wav(&PcmAudio::new(48_000, upsampled).unwrap()).unwrap();
        assert_eq!(decode_text_wav_reliable(&wav).unwrap(), "48 kHz");
    }
}
