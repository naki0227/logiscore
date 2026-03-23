use std::fs;
use std::path::Path;
use harmonic_core::protocol::Header;
use harmonic_core::protocol::midi_gen::{encode_project_to_midi, decode_project_from_midi};
use harmonic_core::compressor;
use harmonic_core::dispatcher;

fn collect_files(dir: &Path, files: &mut Vec<(String, Vec<u8>, Header)>) {
    if !dir.is_dir() { return; }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            // SKIP huge dirs
            if name == "node_modules" || name == ".git" || name == "build" || name == "dist" {
                continue;
            }
            if path.is_dir() {
                collect_files(&path, files);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if ["js", "ts", "jsx", "tsx", "json", "md", "css", "html", "json", "js"].contains(&ext_str.as_ref()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let rel_path = path.strip_prefix("/Users/hw24a094/react").unwrap().to_string_lossy().to_string();
                            let compressed = compressor::compress(content.as_bytes()).unwrap();
                            let mut header = dispatcher::header_for_extension(&ext_str).unwrap();
                            
                            let target_ticks = 200;
                            let bpt = if compressed.is_empty() { 1 } else { (compressed.len() + target_ticks - 1) / target_ticks };
                            header.bytes_per_tick = bpt.clamp(1, 255) as u8;
                            
                            files.push((rel_path, compressed, header));
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    println!("Scanning /Users/hw24a094/react...");
    let mut files = Vec::new();
    collect_files(Path::new("/Users/hw24a094/react"), &mut files);
    
    println!("Found {} files", files.len());
    if files.is_empty() {
        println!("No files found. Make sure the path is correct.");
        return;
    }
    
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let global_header = Header::new(0, 0, 8).unwrap();
    
    println!("Encoding project to MIDI...");
    let midi_bytes = encode_project_to_midi(&files, &global_header).expect("Encode failed");
    println!("Generated MIDI size: {} bytes ({:.2} MB)", midi_bytes.len(), midi_bytes.len() as f64 / 1_048_576.0);
    
    println!("Decoding project from MIDI...");
    let decoded = decode_project_from_midi(&midi_bytes).expect("Decode failed");
    println!("Decoded {} files", decoded.len());
    
    assert_eq!(files.len(), decoded.len(), "File count mismatch!");
    
    println!("Re-encoding project from decoded files to verify bit-perfect roundtrip...");
    // &Vec<(String, Header, Vec<u8>)> を &[(String, Vec<u8>, Header)] に変換する必要がある
    let reencode_input: Vec<(String, Vec<u8>, Header)> = decoded.iter().map(|(name, header, data)| (name.clone(), data.clone(), *header)).collect();
    let reencoded_midi_bytes = encode_project_to_midi(&reencode_input, &global_header).expect("Re-encode failed");
    
    if midi_bytes == reencoded_midi_bytes {
        println!("✅ RE-ENCODED MIDI MATCHES EXACTLY (BIT-PERFECT ROUNDTRIP SUCCESS)");
    } else {
        println!("❌ RE-ENCODED MIDI IS DIFFERENT! Length 1: {}, Length 2: {}", midi_bytes.len(), reencoded_midi_bytes.len());
        // Find first differing byte
        for i in 0..usize::min(midi_bytes.len(), reencoded_midi_bytes.len()) {
            if midi_bytes[i] != reencoded_midi_bytes[i] {
                println!("First byte mismatch at index {}: {:02X} != {:02X}", i, midi_bytes[i], reencoded_midi_bytes[i]);
                break;
            }
        }
    }
    
    let mut mismatch_count = 0;
    for (i, ((orig_name, orig_comp, orig_header), (dec_name, dec_header, dec_comp))) in files.iter().zip(decoded.iter()).enumerate() {
        if orig_name != dec_name {
            println!("Mismatch name at {}: expected {}, got {}", i, orig_name, dec_name);
            mismatch_count += 1;
            continue;
        }
        if orig_header != dec_header {
            println!("Mismatch header at {}: expected {:?}, got {:?}", orig_name, orig_header, dec_header);
            mismatch_count += 1;
            continue;
        }
        if orig_comp != dec_comp {
            println!("Mismatch data at {}: expected len {}, got len {}", orig_name, orig_comp.len(), dec_comp.len());
            let dec_decomp = compressor::decompress(dec_comp);
            if dec_decomp.is_err() {
                println!("  -> Decompression failed: {:?}", dec_decomp.err());
            }
            mismatch_count += 1;
            continue;
        }
    }
    
    if mismatch_count == 0 {
        println!("✅ SUCCESS! All {} files round-tripped perfectly (lossless).", files.len());
        println!("The bug has been completely eradicated.");
    } else {
        println!("❌ FAILED with {} mismatches.", mismatch_count);
    }
}
