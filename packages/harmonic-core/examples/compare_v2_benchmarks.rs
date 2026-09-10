use harmonic_core::benchmark::{compare_reports, BenchmarkReport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let baseline_path = std::env::args()
        .nth(1)
        .ok_or("baseline JSON path is required")?;
    let current_path = std::env::args()
        .nth(2)
        .ok_or("current JSON path is required")?;
    let baseline: BenchmarkReport = serde_json::from_slice(&std::fs::read(baseline_path)?)?;
    let current: BenchmarkReport = serde_json::from_slice(&std::fs::read(current_path)?)?;
    let comparison = compare_reports(&baseline, &current)?;
    println!("{}", serde_json::to_string_pretty(&comparison)?);
    if comparison.functional_regression {
        return Err("functional benchmark regression detected".into());
    }
    Ok(())
}
