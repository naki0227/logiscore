use wasm_bindgen::prelude::*;

use crate::error::LogiscoreError;
use crate::project_payload::ProjectFilePayload;
use crate::{v2, ProjectFile};

fn js_error(error: LogiscoreError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
pub fn encode_text_v2_wav_reliable_wasm(text: &str) -> Result<Vec<u8>, JsValue> {
    v2::encode_text_wav_reliable(text).map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_wav_reliable_wasm(wav_bytes: &[u8]) -> Result<String, JsValue> {
    v2::decode_text_wav_reliable(wav_bytes).map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_recorded_pcm_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    v2::decode_text_pcm_reliable_at_sample_rate(samples, sample_rate).map_err(js_error)
}

#[wasm_bindgen]
pub fn encode_text_v2_checkpoint_loop_pcm_wasm(
    text: &str,
    chunk_payload_bytes: u32,
    loops: u32,
) -> Result<Vec<f32>, JsValue> {
    let chunk_payload_bytes = usize::try_from(chunk_payload_bytes)
        .map_err(|_| JsValue::from_str("checkpoint chunk size is invalid"))?;
    let loops = usize::try_from(loops)
        .map_err(|_| JsValue::from_str("checkpoint loop count is invalid"))?;
    v2::checkpoint::encode_text_checkpoint_loop(text, chunk_payload_bytes, loops).map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_checkpoint_loop_pcm_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    v2::checkpoint::decode_text_checkpoint_loop_recording(samples, sample_rate).map_err(js_error)
}

#[wasm_bindgen]
pub fn encode_source_file_v2_wav_reliable_wasm(
    filename: &str,
    extension: &str,
    source: &str,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_source_file_wav_reliable(filename, extension, source).map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_source_file_v2_wav_reliable_wasm(wav_bytes: &[u8]) -> Result<String, JsValue> {
    let payload = v2::decode_source_file_wav_reliable(wav_bytes).map_err(js_error)?;
    Ok(serde_json::json!({
        "filename": payload.filename(),
        "extension": payload.extension(),
        "source": payload.source(),
    })
    .to_string())
}

#[wasm_bindgen]
pub fn decode_source_file_v2_recorded_pcm_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    let payload = v2::decode_source_file_pcm_reliable_at_sample_rate(samples, sample_rate)
        .map_err(js_error)?;
    Ok(serde_json::json!({
        "filename": payload.filename(),
        "extension": payload.extension(),
        "source": payload.source(),
    })
    .to_string())
}

#[wasm_bindgen]
pub fn encode_project_v2_wav_reliable_wasm(input_json: &str) -> Result<Vec<u8>, JsValue> {
    let files = parse_project_files(input_json)?;
    v2::encode_project_wav_reliable(files).map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_project_v2_wav_reliable_wasm(wav_bytes: &[u8]) -> Result<String, JsValue> {
    let payload = v2::decode_project_wav_reliable(wav_bytes).map_err(js_error)?;
    serialize_project(payload)
}

#[wasm_bindgen]
pub fn decode_project_v2_recorded_pcm_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    let payload =
        v2::decode_project_pcm_reliable_at_sample_rate(samples, sample_rate).map_err(js_error)?;
    serialize_project(payload)
}

pub(crate) fn parse_project_files(input_json: &str) -> Result<Vec<ProjectFilePayload>, JsValue> {
    let files: Vec<ProjectFile> = serde_json::from_str(input_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid project JSON: {error}")))?;
    files
        .into_iter()
        .map(|file| ProjectFilePayload::new(file.name, file.extension, file.source))
        .collect::<Result<Vec<_>, _>>()
        .map_err(js_error)
}

pub(crate) fn serialize_project(
    payload: crate::project_payload::ProjectPayload,
) -> Result<String, JsValue> {
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
