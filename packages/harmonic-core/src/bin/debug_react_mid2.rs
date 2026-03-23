use std::fs;

fn main() {
    let raw = fs::read("../../react.mid").expect("Failed to read react.mid");
    let projects = harmonic_core::protocol::midi_gen::decode_project_from_midi(&raw).unwrap();
    
    let mut empty_count = 0;
    
    for (name, _header, data) in &projects {
        match harmonic_core::compressor::decompress(data) {
            Ok(decompressed) => {
                if decompressed.is_empty() && !data.is_empty() {
                    empty_count += 1;
                }
            }
            Err(e) => {
                println!("File {}: Decompress error: {}, data len: {}", name, e, data.len());
                empty_count += 1;
            }
        }
    }
    
    println!("Total files: {}", projects.len());
    println!("Empty files or errors: {}", empty_count);
}
