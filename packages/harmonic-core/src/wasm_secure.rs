use wasm_bindgen::prelude::*;

use crate::error::LogiscoreError;
use crate::v2;
use crate::wasm_adaptive::{parse_environment, serialize_source_file};
use crate::wasm_audio::{parse_project_files, serialize_project};

#[wasm_bindgen]
pub fn decode_v2_wav_secure_wasm(bytes: &[u8], password: &str) -> Result<String, JsValue> {
    serialize_secure_payload(
        v2::decode_wav_secure_auto(bytes, password)
            .map_err(js_error)?
            .payload,
    )
}

#[wasm_bindgen]
pub fn decode_v2_recorded_pcm_secure_wasm(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<String, JsValue> {
    serialize_secure_payload(
        v2::decode_pcm_secure_auto(samples, sample_rate, password)
            .map_err(js_error)?
            .payload,
    )
}

#[wasm_bindgen]
pub fn encode_text_v2_wav_secure_wasm(
    text: &str,
    password: &str,
    environment: &str,
    reliability_priority: u8,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_text_wav_secure(
        text,
        password,
        parse_environment(environment)?,
        reliability_priority,
    )
    .map(|encoded| encoded.bytes)
    .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_wav_secure_wasm(bytes: &[u8], password: &str) -> Result<String, JsValue> {
    v2::decode_text_wav_secure(bytes, password)
        .map(|decoded| decoded.payload)
        .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_recorded_pcm_secure_wasm(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<String, JsValue> {
    v2::decode_text_pcm_secure_at_sample_rate(samples, sample_rate, password)
        .map(|decoded| decoded.payload)
        .map_err(js_error)
}

#[wasm_bindgen]
pub fn encode_source_file_v2_wav_secure_wasm(
    filename: &str,
    extension: &str,
    source: &str,
    password: &str,
    environment: &str,
    reliability_priority: u8,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_source_file_wav_secure(
        filename,
        extension,
        source,
        password,
        parse_environment(environment)?,
        reliability_priority,
    )
    .map(|encoded| encoded.bytes)
    .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_source_file_v2_wav_secure_wasm(
    bytes: &[u8],
    password: &str,
) -> Result<String, JsValue> {
    let decoded = v2::decode_source_file_wav_secure(bytes, password).map_err(js_error)?;
    serialize_source_file(decoded.payload)
}

#[wasm_bindgen]
pub fn decode_source_file_v2_recorded_pcm_secure_wasm(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<String, JsValue> {
    let decoded = v2::decode_source_file_pcm_secure_at_sample_rate(samples, sample_rate, password)
        .map_err(js_error)?;
    serialize_source_file(decoded.payload)
}

#[wasm_bindgen]
pub fn encode_project_v2_wav_secure_wasm(
    input_json: &str,
    password: &str,
    environment: &str,
    reliability_priority: u8,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_project_wav_secure(
        parse_project_files(input_json)?,
        password,
        parse_environment(environment)?,
        reliability_priority,
    )
    .map(|encoded| encoded.bytes)
    .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_project_v2_wav_secure_wasm(bytes: &[u8], password: &str) -> Result<String, JsValue> {
    let decoded = v2::decode_project_wav_secure(bytes, password).map_err(js_error)?;
    serialize_project(decoded.payload)
}

#[wasm_bindgen]
pub fn decode_project_v2_recorded_pcm_secure_wasm(
    samples: &[f32],
    sample_rate: u32,
    password: &str,
) -> Result<String, JsValue> {
    let decoded = v2::decode_project_pcm_secure_at_sample_rate(samples, sample_rate, password)
        .map_err(js_error)?;
    serialize_project(decoded.payload)
}

fn js_error(error: LogiscoreError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn serialize_secure_payload(payload: v2::SecurePayload) -> Result<String, JsValue> {
    let value = match payload {
        v2::SecurePayload::Text(text) => serde_json::json!({
            "type": "text",
            "text": text,
        }),
        v2::SecurePayload::SourceFile(file) => serde_json::json!({
            "type": "source-file",
            "version": 2,
            "filename": file.filename(),
            "extension": file.extension(),
            "source": file.source(),
        }),
        v2::SecurePayload::Project(project) => {
            let files = project
                .files()
                .iter()
                .map(|file| {
                    serde_json::json!({
                        "name": file.path(),
                        "extension": file.extension(),
                        "source": file.source(),
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "type": "project",
                "version": 2,
                "files": files,
            })
        }
    };
    serde_json::to_string(&value)
        .map_err(|_| JsValue::from_str("Secure payload serialization failed"))
}
