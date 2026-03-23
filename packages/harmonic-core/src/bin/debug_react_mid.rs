use std::fs;

fn main() {
    let raw = fs::read("../../react.mid").expect("Failed to read react.mid");
    println!("File size: {} bytes", raw.len());
    
    // Call harmonic-core logic directly
    match harmonic_core::protocol::midi_gen::decode_project_from_midi(&raw) {
        Ok(res) => {
            println!("Decoded {} files", res.len());
        }
        Err(e) => {
            println!("Error decoding: {}", e);
        }
    }
}
