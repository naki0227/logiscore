use harmonic_core::adaptive::AcousticProfileId;
use harmonic_core::payload::PayloadType;
use harmonic_core::v2::encode_wav_with_profile;

const OPUS_FIXTURE_TEXT: &str = "Opus";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("output WAV path is required")?;
    let wav = encode_wav_with_profile(
        PayloadType::Text,
        OPUS_FIXTURE_TEXT.as_bytes(),
        AcousticProfileId::FixedFallback,
    )?;
    std::fs::write(output, wav)?;
    Ok(())
}
