use harmonic_core::v2::decode_text_wav_adaptive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or("input WAV path is required")?;
    let wav = std::fs::read(input)?;
    let decoded = decode_text_wav_adaptive(&wav)?;
    println!("{}", decoded.payload);
    Ok(())
}
