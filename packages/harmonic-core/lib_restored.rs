pub mod compressor;
pub mod dispatcher;
pub mod error;
pub mod protocol;

use error::LogiscoreError;
use wasm_bindgen::prelude::*;

// --- WASM エントリーポイント ---

/// ソースコードを MIDI バイナリにエンコードする（WASM 公開 API）。
#[wasm_bindgen]
pub fn encode_wasm(source: &str, extension: &str) -> Result<Vec<u8>, JsValue> {
    encode(source, extension).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// MIDI バイナリをソースコードにデコードする（WASM 公開 API）。
#[wasm_bindgen]
pub fn decode_wasm(midi_bytes: &[u8]) -> Result<String, JsValue> {
    decode(midi_bytes).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 拡張子のメタ情報を JSON 文字列で返す（WASM 公開 API）。
#[wasm_bindgen]
pub fn get_extension_info(extension: &str) -> String {
    let info = dispatcher::dispatch(extension);
    serde_json::json!({
        "scale_id": info.scale_id,
        "root_key": info.root_key,
        "name": info.name,
        "scale_name": protocol::scales::SCALE_NAMES[info.scale_id as usize],
    })
    .to_string()
}

// --- Rust ネイティブ API ---

/// ソースコードを MIDI バイナリにエンコードする。
///
/// # Arguments
/// * `source` - ソースコード文字列
/// * `extension` - ファイル拡張子 (例: ".rs", ".py")
///
/// # Returns
/// SMF (Standard MIDI File) バイナリの `Vec<u8>`。
///
/// # Errors
/// 圧縮失敗、ヘッダー生成失敗、MIDI構築失敗時にエラーを返す。
/// データの長さから最適な密度 (1拍あたりのバイト数) を算出する。
/// 目標演奏時間は 200拍 (約50秒)。
fn calculate_optimal_density(data_len: usize) -> u8 {
    if data_len == 0 { return 1; }
    let target_ticks = 200;
    let bpt = (data_len + target_ticks - 1) / target_ticks;
    bpt.clamp(1, 255) as u8
}

/// ソースコードを MIDI バイナリにエンコードする。
pub fn encode(source: &str, extension: &str) -> Result<Vec<u8>, LogiscoreError> {
    // 1. 圧縮
    let compressed = compressor::compress(source.as_bytes())?;

    // 2. 拡張子 → Header 決定
    let mut header = dispatcher::header_for_extension(extension)?;
    
    // 3. 密度を動的に最適化
    header.bytes_per_tick = calculate_optimal_density(compressed.len());

    // 4. MIDI 生成
    protocol::midi_gen::encode_to_midi(&compressed, &header)
}

/// MIDI バイナリをソースコードにデコードする。
///
/// # Arguments
/// * `midi_bytes` - SMF バイナリ
///
/// # Returns
/// 復元されたソースコード文字列。
///
/// # Errors
/// MIDI解析失敗、バイト復元失敗、展開失敗、UTF-8 変換失敗時にエラーを返す。
pub fn decode(midi_bytes: &[u8]) -> Result<String, LogiscoreError> {
    // 1. MIDI 解析 & バイト復元
    let (_header, compressed) = protocol::midi_gen::decode_from_midi(midi_bytes)?;

    // 2. 展開
    let decompressed = compressor::decompress(&compressed)?;

    // 3. UTF-8 文字列に変換
    String::from_utf8(decompressed).map_err(LogiscoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_roundtrip_hello() {
        let source = "fn main() { println!(\"Hello, world!\"); }";
        let midi = encode(source, ".rs").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_empty() {
        let source = "";
        let midi = encode(source, ".rs").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_single_char() {
        let source = "x";
        let midi = encode(source, ".py").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_multiline() {
        let source = r#"
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    println!("You said: {}", input.trim());
}
"#;
        let midi = encode(source, ".rs").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_python() {
        let source = "def hello():\n    print('Hello, world!')\n\nhello()\n";
        let midi = encode(source, ".py").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_typescript() {
        let source = "const greet = (name: string): string => `Hello, ${name}!`;\nconsole.log(greet('World'));\n";
        let midi = encode(source, ".ts").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_json() {
        let source = r#"{"name": "logiscore", "version": "1.0.0", "dependencies": {}}"#;
        let midi = encode(source, ".json").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_go() {
        let source = "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello\")\n}\n";
        let midi = encode(source, ".go").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_1000_lines() {
        // 1000行のコードで可逆性テスト
        let mut source = String::new();
        for i in 0..1000 {
            source.push_str(&format!("let var_{} = {};\n", i, i * 42));
        }
        let midi = encode(&source, ".rs").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded, "1000-line roundtrip failed");
    }

    #[test]
    fn full_roundtrip_unicode() {
        let source = "// こんにちは世界 🌍\nfn greet() -> &'static str { \"日本語\" }\n";
        let midi = encode(source, ".rs").unwrap();
        let decoded = decode(&midi).unwrap();
        assert_eq!(source, decoded);
    }

    #[test]
    fn full_roundtrip_all_extensions() {
        let source = "hello world test data for encoding";
        for ext in &[".rs", ".py", ".ts", ".go", ".json", ".yaml", ".yml", ".html"] {
            let midi = encode(source, ext).unwrap();
            let decoded = decode(&midi).unwrap();
            assert_eq!(source, decoded, "Failed for extension {}", ext);
        }
    }

    #[test]
    fn midi_binary_is_valid_smf() {
        let source = "fn main() {}";
        let midi = encode(source, ".rs").unwrap();
        // midly がパースできることで SMF として有効であることを確認
        assert!(midly::Smf::parse(&midi).is_ok());
    }
}
