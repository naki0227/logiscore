use harmonic_core::adaptive::AcousticProfileId;
use harmonic_core::payload::PayloadType;
use harmonic_core::v2::encode_wav_with_profile;

const OPUS_FIXTURE_TEXT: &str = "Opus";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let output = arguments.next().ok_or("output WAV path is required")?;
    let text = arguments
        .next()
        .unwrap_or_else(|| OPUS_FIXTURE_TEXT.to_owned());
    if arguments.next().is_some() {
        return Err("only output path and optional Text payload are supported".into());
    }
    let wav = encode_wav_with_profile(
        PayloadType::Text,
        text.as_bytes(),
        AcousticProfileId::FixedFallback,
    )?;
    std::fs::write(output, wav)?;
    Ok(())
}
