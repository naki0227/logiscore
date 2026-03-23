use std::fs;
use harmonic_core::{encode_project_wasm, decode_project_wasm};

fn main() {
    println!("Testing WASM bridge functionality...");
    
    let mut files = Vec::new();
    let json_input = r#"[
        {"name": "test.txt", "source": "hello world", "extension": ".txt"},
        {"name": "test.rs", "source": "fn main() { println!(\"こんにちは🌍\"); }", "extension": ".rs"}
    ]"#;
    
    println!("Calling encode_project_wasm...");
    let midi_bytes = encode_project_wasm(json_input).expect("Failed encode_project_wasm");
    println!("Generated {} bytes.", midi_bytes.len());
    
    println!("Calling decode_project_wasm...");
    let result_json = decode_project_wasm(&midi_bytes).expect("Failed decode_project_wasm");
    // decode_project_wasm returns JsValue string in WASM, but here we compile it with #[wasm_bindgen] disabled or something?
}
