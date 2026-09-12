use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::adaptive::{estimate_payload_duration_ms, select_profile, Environment, SelectionInput};
use crate::error::LogiscoreError;
use crate::v2;
use crate::wasm_audio::{parse_project_files, serialize_project};

#[derive(Serialize)]
struct ProfileDescription {
    id: u8,
    name: &'static str,
    detected_environment: &'static str,
    confidence_percent: u8,
    timing_percent: u16,
    max_polyphony: u8,
    fec_profile: u8,
    interleave_depth: u8,
    repetition: u8,
    music_weight: u8,
    estimated_duration_ms: u64,
    fallback: Vec<u8>,
}

#[wasm_bindgen]
pub fn describe_acoustic_profile_wasm(
    environment: &str,
    reliability_priority: u8,
    payload_bytes: usize,
) -> Result<String, JsValue> {
    let selection = select_profile(SelectionInput {
        environment: parse_environment(environment)?,
        reliability_priority,
        payload_bytes,
        calibration: None,
    })
    .map_err(js_error)?;
    serde_json::to_string(&ProfileDescription {
        id: selection.primary.id as u8,
        name: profile_name(selection.primary.id),
        detected_environment: environment_name(selection.detected_environment),
        confidence_percent: selection.confidence_percent,
        timing_percent: selection.primary.timing_percent,
        max_polyphony: selection.primary.max_polyphony,
        fec_profile: selection.primary.fec_profile,
        interleave_depth: selection.primary.interleave_depth,
        repetition: selection.primary.repetition,
        music_weight: selection.primary.music_weight,
        estimated_duration_ms: estimate_payload_duration_ms(selection.primary, payload_bytes),
        fallback: selection.fallback.iter().map(|id| *id as u8).collect(),
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn encode_text_v2_wav_adaptive_wasm(
    text: &str,
    environment: &str,
    reliability_priority: u8,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_text_wav_adaptive(text, parse_environment(environment)?, reliability_priority)
        .map(|encoded| encoded.bytes)
        .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_wav_adaptive_wasm(bytes: &[u8]) -> Result<String, JsValue> {
    v2::decode_text_wav_adaptive(bytes)
        .map(|decoded| decoded.payload)
        .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_text_v2_recorded_pcm_adaptive_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    v2::decode_text_pcm_adaptive_at_sample_rate(samples, sample_rate)
        .map(|decoded| decoded.payload)
        .map_err(js_error)
}

#[wasm_bindgen]
pub fn analyze_text_v2_recorded_pcm_wasm(
    samples: &[f32],
    sample_rate: u32,
    expected_text: &str,
) -> Result<String, JsValue> {
    let telemetry = crate::benchmark::analyze_text_recording(samples, sample_rate, expected_text)
        .map_err(js_error)?;
    serde_json::to_string(&telemetry).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn encode_source_file_v2_wav_adaptive_wasm(
    filename: &str,
    extension: &str,
    source: &str,
    environment: &str,
    reliability_priority: u8,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_source_file_wav_adaptive(
        filename,
        extension,
        source,
        parse_environment(environment)?,
        reliability_priority,
    )
    .map(|encoded| encoded.bytes)
    .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_source_file_v2_wav_adaptive_wasm(bytes: &[u8]) -> Result<String, JsValue> {
    let decoded = v2::decode_source_file_wav_adaptive(bytes).map_err(js_error)?;
    serialize_source_file(decoded.payload)
}

#[wasm_bindgen]
pub fn decode_source_file_v2_recorded_pcm_adaptive_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    let decoded = v2::decode_source_file_pcm_adaptive_at_sample_rate(samples, sample_rate)
        .map_err(js_error)?;
    serialize_source_file(decoded.payload)
}

#[wasm_bindgen]
pub fn encode_project_v2_wav_adaptive_wasm(
    input_json: &str,
    environment: &str,
    reliability_priority: u8,
) -> Result<Vec<u8>, JsValue> {
    v2::encode_project_wav_adaptive(
        parse_project_files(input_json)?,
        parse_environment(environment)?,
        reliability_priority,
    )
    .map(|encoded| encoded.bytes)
    .map_err(js_error)
}

#[wasm_bindgen]
pub fn decode_project_v2_wav_adaptive_wasm(bytes: &[u8]) -> Result<String, JsValue> {
    let decoded = v2::decode_project_wav_adaptive(bytes).map_err(js_error)?;
    serialize_project(decoded.payload)
}

#[wasm_bindgen]
pub fn decode_project_v2_recorded_pcm_adaptive_wasm(
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, JsValue> {
    let decoded =
        v2::decode_project_pcm_adaptive_at_sample_rate(samples, sample_rate).map_err(js_error)?;
    serialize_project(decoded.payload)
}

pub(crate) fn parse_environment(value: &str) -> Result<Environment, JsValue> {
    match value {
        "auto" => Ok(Environment::Auto),
        "quiet" => Ok(Environment::Quiet),
        "conversation" => Ok(Environment::Conversation),
        "noisy" => Ok(Environment::Noisy),
        "online" => Ok(Environment::Online),
        "long-distance" => Ok(Environment::LongDistance),
        _ => Err(JsValue::from_str("Unsupported acoustic environment")),
    }
}

pub(crate) fn serialize_source_file(
    payload: crate::payload::SourceFilePayload,
) -> Result<String, JsValue> {
    serde_json::to_string(&serde_json::json!({
        "filename": payload.filename(),
        "extension": payload.extension(),
        "source": payload.source(),
    }))
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

fn profile_name(id: crate::adaptive::AcousticProfileId) -> &'static str {
    use crate::adaptive::AcousticProfileId::*;
    match id {
        Quiet => "Quiet",
        Balanced => "Balanced",
        Conversation => "Conversation",
        Noisy => "Noisy",
        Online => "Online",
        LongDistance => "Long Distance",
        FixedFallback => "Fixed Fallback",
    }
}

fn environment_name(environment: Environment) -> &'static str {
    match environment {
        Environment::Auto => "Auto",
        Environment::Quiet => "Quiet",
        Environment::Conversation => "Conversation",
        Environment::Noisy => "Noisy",
        Environment::Online => "Online",
        Environment::LongDistance => "Long Distance",
    }
}

fn js_error(error: LogiscoreError) -> JsValue {
    JsValue::from_str(&error.to_string())
}
