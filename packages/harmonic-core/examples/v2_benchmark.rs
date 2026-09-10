use harmonic_core::benchmark::run_text_benchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = parse_arg(1, 256)?;
    let iterations = parse_arg(2, 3)?;
    let fixture = deterministic_fixture(size);
    let report = run_text_benchmark(&fixture, "benchmark-only-password", iterations)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn parse_arg(index: usize, default: usize) -> Result<usize, Box<dyn std::error::Error>> {
    std::env::args()
        .nth(index)
        .map(|value| value.parse().map_err(Into::into))
        .unwrap_or(Ok(default))
}

fn deterministic_fixture(size: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
    (0..size)
        .map(|index| ALPHABET[(index * 17 + index / 7) % ALPHABET.len()] as char)
        .collect()
}
