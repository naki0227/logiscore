use wasm_bindgen::prelude::*;

use crate::project_payload::ProjectFilePayload;
use crate::{error::LogiscoreError, protocol, v2, ProjectFile};

fn js_error(error: LogiscoreError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// Text Payloadをv2 Binary Header + Dense MIDIでエンコードする。
#[wasm_bindgen]
pub fn encode_text_v2_wasm(text: &str) -> Result<Vec<u8>, JsValue> {
    v2::encode_text_dense(text).map_err(js_error)
}

/// v2 Text PayloadをDense MIDIから復元する。
#[wasm_bindgen]
pub fn decode_text_v2_wasm(midi_bytes: &[u8]) -> Result<String, JsValue> {
    v2::decode_text_dense(midi_bytes)
        .or_else(|_| v2::decode_text_rhythmic(midi_bytes))
        .or_else(|_| v2::decode_text_musical(midi_bytes))
        .map_err(js_error)
}

/// Text Payloadをv2 Musical MIDIでエンコードする。
#[wasm_bindgen]
pub fn encode_text_v2_musical_wasm(text: &str) -> Result<Vec<u8>, JsValue> {
    v2::encode_text_rhythmic(text).map_err(js_error)
}

/// Source File Payloadをv2 Binary Header + Dense MIDIでエンコードする。
#[wasm_bindgen]
pub fn encode_source_file_v2_wasm(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_source_file_dense(filename, extension, source).map_err(js_error)
}

/// v2 Source File PayloadをDense MIDIから復元する。
#[wasm_bindgen]
pub fn decode_source_file_v2_wasm(midi_bytes: &[u8]) -> Result<String, JsValue> {
    let payload = v2::decode_source_file_dense(midi_bytes)
        .or_else(|_| v2::decode_source_file_rhythmic(midi_bytes))
        .or_else(|_| v2::decode_source_file_musical(midi_bytes))
        .map_err(js_error)?;
    Ok(serde_json::json!({
        "filename": payload.filename(),
        "extension": payload.extension(),
        "source": payload.source(),
    })
    .to_string())
}

/// Source File Payloadをv2 Musical MIDIでエンコードする。
#[wasm_bindgen]
pub fn encode_source_file_v2_musical_wasm(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_source_file_rhythmic(filename, extension, source).map_err(js_error)
}

/// Project Payloadをv2 Binary Header + Dense MIDIでエンコードする。
#[wasm_bindgen]
pub fn encode_project_v2_wasm(input_json: &str) -> Result<Vec<u8>, JsValue> {
    let files: Vec<ProjectFile> = serde_json::from_str(input_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid project JSON: {error}")))?;
    let payload_files = files
        .into_iter()
        .map(|file| ProjectFilePayload::new(file.name, file.extension, file.source))
        .collect::<Result<Vec<_>, _>>()
        .map_err(js_error)?;
    v2::encode_project_dense(payload_files).map_err(js_error)
}

/// v2 Project PayloadをDense MIDIから復元する。
#[wasm_bindgen]
pub fn decode_project_v2_wasm(midi_bytes: &[u8]) -> Result<String, JsValue> {
    let payload = v2::decode_project_dense(midi_bytes)
        .or_else(|_| v2::decode_project_rhythmic(midi_bytes))
        .or_else(|_| v2::decode_project_musical(midi_bytes))
        .map_err(js_error)?;
    let files = payload
        .files()
        .iter()
        .map(|file| ProjectFile {
            name: file.path().to_owned(),
            extension: file.extension().to_owned(),
            source: file.source().to_owned(),
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&files).map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Project Payloadをv2 Musical MIDIでエンコードする。
#[wasm_bindgen]
pub fn encode_project_v2_musical_wasm(input_json: &str) -> Result<Vec<u8>, JsValue> {
    let files: Vec<ProjectFile> = serde_json::from_str(input_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid project JSON: {error}")))?;
    let payload_files = files
        .into_iter()
        .map(|file| ProjectFilePayload::new(file.name, file.extension, file.source))
        .collect::<Result<Vec<_>, _>>()
        .map_err(js_error)?;
    v2::encode_project_rhythmic(payload_files).map_err(js_error)
}

/// v2 packet protocolのバージョンを返す。
#[wasm_bindgen]
pub fn get_v2_version() -> u8 {
    protocol::v2::PROTOCOL_VERSION
}

/// Text Payloadをclean PCMへエンコードする。
#[wasm_bindgen]
pub fn encode_text_v2_pcm_wasm(text: &str) -> Result<Vec<f32>, JsValue> {
    v2::encode_text_pcm(text).map_err(js_error)
}

/// clean PCMからv2 Text Payloadを復元する。
#[wasm_bindgen]
pub fn decode_text_v2_pcm_wasm(samples: &[f32]) -> Result<String, JsValue> {
    v2::decode_text_pcm(samples).map_err(js_error)
}

/// Text PayloadをCRC/FEC付きPCMへエンコードする。
#[wasm_bindgen]
pub fn encode_text_v2_pcm_reliable_wasm(text: &str) -> Result<Vec<f32>, JsValue> {
    v2::encode_text_pcm_reliable(text).map_err(js_error)
}

/// CRC/FEC付きPCMからv2 Text Payloadを復元する。
#[wasm_bindgen]
pub fn decode_text_v2_pcm_reliable_wasm(samples: &[f32]) -> Result<String, JsValue> {
    v2::decode_text_pcm_reliable(samples).map_err(js_error)
}

/// Source File Payloadをclean PCMへエンコードする。
#[wasm_bindgen]
pub fn encode_source_file_v2_pcm_wasm(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<f32>, JsValue> {
    v2::encode_source_file_pcm(filename, extension, source).map_err(js_error)
}

/// clean PCMからv2 Source File Payloadを復元する。
#[wasm_bindgen]
pub fn decode_source_file_v2_pcm_wasm(samples: &[f32]) -> Result<String, JsValue> {
    let payload = v2::decode_source_file_pcm(samples).map_err(js_error)?;
    Ok(serde_json::json!({
        "filename": payload.filename(),
        "extension": payload.extension(),
        "source": payload.source(),
    })
    .to_string())
}

/// Source File PayloadをCRC/FEC付きPCMへエンコードする。
#[wasm_bindgen]
pub fn encode_source_file_v2_pcm_reliable_wasm(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<f32>, JsValue> {
    v2::encode_source_file_pcm_reliable(filename, extension, source).map_err(js_error)
}

/// CRC/FEC付きPCMからv2 Source File Payloadを復元する。
#[wasm_bindgen]
pub fn decode_source_file_v2_pcm_reliable_wasm(samples: &[f32]) -> Result<String, JsValue> {
    let payload = v2::decode_source_file_pcm_reliable(samples).map_err(js_error)?;
    Ok(serde_json::json!({
        "filename": payload.filename(),
        "extension": payload.extension(),
        "source": payload.source(),
    })
    .to_string())
}

/// Project Payloadをclean PCMへエンコードする。
#[wasm_bindgen]
pub fn encode_project_v2_pcm_wasm(input_json: &str) -> Result<Vec<f32>, JsValue> {
    let files: Vec<ProjectFile> = serde_json::from_str(input_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid project JSON: {error}")))?;
    let payload_files = files
        .into_iter()
        .map(|file| ProjectFilePayload::new(file.name, file.extension, file.source))
        .collect::<Result<Vec<_>, _>>()
        .map_err(js_error)?;
    v2::encode_project_pcm(payload_files).map_err(js_error)
}

/// clean PCMからv2 Project Payloadを復元する。
#[wasm_bindgen]
pub fn decode_project_v2_pcm_wasm(samples: &[f32]) -> Result<String, JsValue> {
    let payload = v2::decode_project_pcm(samples).map_err(js_error)?;
    let files = payload
        .files()
        .iter()
        .map(|file| ProjectFile {
            name: file.path().to_owned(),
            extension: file.extension().to_owned(),
            source: file.source().to_owned(),
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&files).map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Project PayloadをCRC/FEC付きPCMへエンコードする。
#[wasm_bindgen]
pub fn encode_project_v2_pcm_reliable_wasm(input_json: &str) -> Result<Vec<f32>, JsValue> {
    let files: Vec<ProjectFile> = serde_json::from_str(input_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid project JSON: {error}")))?;
    let payload_files = files
        .into_iter()
        .map(|file| ProjectFilePayload::new(file.name, file.extension, file.source))
        .collect::<Result<Vec<_>, _>>()
        .map_err(js_error)?;
    v2::encode_project_pcm_reliable(payload_files).map_err(js_error)
}

/// CRC/FEC付きPCMからv2 Project Payloadを復元する。
#[wasm_bindgen]
pub fn decode_project_v2_pcm_reliable_wasm(samples: &[f32]) -> Result<String, JsValue> {
    let payload = v2::decode_project_pcm_reliable(samples).map_err(js_error)?;
    let files = payload
        .files()
        .iter()
        .map(|file| ProjectFile {
            name: file.path().to_owned(),
            extension: file.extension().to_owned(),
            source: file.source().to_owned(),
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&files).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn get_pcm_sample_rate() -> u32 {
    v2::pcm_sample_rate()
}
