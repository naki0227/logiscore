use clap::{Parser, Subcommand};
use harmonic_core::{encode, decode};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "logiscore")]
#[command(about = "Logiscore: Reversible source code to MIDI encoder/decoder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a source file to a MIDI file
    Encode {
        /// Path to the source file
        input: PathBuf,
        /// Path to the output MIDI file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decode a MIDI file back to source code
    Decode {
        /// Path to the MIDI file
        input: PathBuf,
        /// Path to the output source file (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode { input, output } => {
            let source = fs::read_to_string(&input)?;
            let extension = input.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("txt");
            
            println!("Encoding {} (ext: {})...", input.display(), extension);
            
            let midi_data = encode(&source, extension)
                .map_err(|e| anyhow::anyhow!("Encoding error: {:?}", e))?;
            
            let out_path = output.unwrap_or_else(|| {
                let mut p = input.clone();
                p.set_extension("mid");
                p
            });
            
            fs::write(&out_path, midi_data)?;
            println!("✅ Exported to: {}", out_path.display());
        }
        Commands::Decode { input, output } => {
            let midi_bytes = fs::read(&input)?;
            
            println!("Decoding {}...", input.display());
            
            let (decoded_text, extension) = decode(&midi_bytes)
                .map_err(|e| anyhow::anyhow!("Decoding error: {:?}", e))?;
            
            println!("Detected Language/Extension: {}", extension);

            match output {
                Some(path) => {
                    fs::write(&path, decoded_text)?;
                    println!("✅ Restored to: {}", path.display());
                }
                None => {
                    println!("--- DECODED SOURCE ({}) ---", extension);
                    println!("{}", decoded_text);
                    println!("---------------------------");
                }
            }
        }
    }

    Ok(())
}
