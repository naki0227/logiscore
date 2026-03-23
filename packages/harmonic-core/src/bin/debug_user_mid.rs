use std::fs;

fn main() {
    println!("Reading user's react.mid...");
    let raw = match fs::read("../../react.mid") {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to read react.mid: {}", e);
            return;
        }
    };
    println!("File size: {} bytes", raw.len());
    
    let projects = match harmonic_core::protocol::midi_gen::decode_project_from_midi(&raw) {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to decode project: {:?}", e);
            return;
        }
    };
    
    let mut empty_decompressed = 0;
    let mut unwrap_failed = 0;
    
    println!("Total files decoded from MIDI: {}", projects.len());
    for (i, (name, _header, data)) in projects.iter().enumerate().take(10) {
        println!("File {}: name: {}, compressed len: {}", i, name, data.len());
        match harmonic_core::compressor::decompress(data) {
            Ok(decomp) => {
                println!("  -> decompressed len: {}", decomp.len());
                if decomp.is_empty() && !data.is_empty() {
                    empty_decompressed += 1;
                }
            }
            Err(e) => {
                println!("  -> DECOMPRESS ERROR: {}", e);
                unwrap_failed += 1;
            }
        }
    }
}
