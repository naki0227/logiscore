import {
  analyze_text_v2_recorded_pcm_wasm,
  decode_project_v2_recorded_pcm_adaptive_wasm,
  decode_project_v2_wav_adaptive_wasm,
  decode_source_file_v2_recorded_pcm_adaptive_wasm,
  decode_source_file_v2_wav_adaptive_wasm,
  decode_text_v2_recorded_pcm_adaptive_wasm,
  decode_text_v2_wav_adaptive_wasm,
  describe_acoustic_profile_wasm,
  encode_project_v2_wav_adaptive_wasm,
  encode_source_file_v2_wav_adaptive_wasm,
  encode_text_v2_wav_adaptive_wasm,
} from "../pkg/harmonic_core.js";
import {
  parseCoreRecordingTelemetry,
  type CoreRecordingTelemetry,
} from "../features/audio/recordingMetricsModel";
import {
  parseProfileDescription,
  type AcousticProfileDescription,
  type AcousticSettings,
} from "../features/acoustic/model";
import type { ProjectFileData, SourceFileV2 } from "./wasm-loader";

export function describeAcousticProfile(
  settings: AcousticSettings,
  payloadBytes: number,
): AcousticProfileDescription {
  return parseProfileDescription(
    describe_acoustic_profile_wasm(
      settings.environment,
      settings.reliabilityPriority,
      payloadBytes,
    ),
  );
}

export function encodeTextV2WavReliable(
  text: string,
  settings: AcousticSettings,
): Uint8Array {
  return encode_text_v2_wav_adaptive_wasm(
    text,
    settings.environment,
    settings.reliabilityPriority,
  );
}

export function decodeTextV2WavReliable(wavBytes: Uint8Array): string {
  return decode_text_v2_wav_adaptive_wasm(wavBytes);
}

export function decodeTextV2RecordedPcm(
  samples: Float32Array,
  sampleRate: number,
): string {
  return decode_text_v2_recorded_pcm_adaptive_wasm(samples, sampleRate);
}

export function analyzeTextV2RecordedPcm(
  samples: Float32Array,
  sampleRate: number,
  expectedText: string,
): CoreRecordingTelemetry {
  return parseCoreRecordingTelemetry(
    analyze_text_v2_recorded_pcm_wasm(samples, sampleRate, expectedText),
  );
}

export function encodeSourceFileV2WavReliable(
  filename: string,
  extension: string,
  source: string,
  settings: AcousticSettings,
): Uint8Array {
  return encode_source_file_v2_wav_adaptive_wasm(
    filename,
    extension,
    source,
    settings.environment,
    settings.reliabilityPriority,
  );
}

export function decodeSourceFileV2WavReliable(
  wavBytes: Uint8Array,
): SourceFileV2 {
  return JSON.parse(
    decode_source_file_v2_wav_adaptive_wasm(wavBytes),
  ) as SourceFileV2;
}

export function decodeSourceFileV2RecordedPcm(
  samples: Float32Array,
  sampleRate: number,
): SourceFileV2 {
  return JSON.parse(
    decode_source_file_v2_recorded_pcm_adaptive_wasm(samples, sampleRate),
  ) as SourceFileV2;
}

export function encodeProjectV2WavReliable(
  files: ProjectFileData[],
  settings: AcousticSettings,
): Uint8Array {
  return encode_project_v2_wav_adaptive_wasm(
    JSON.stringify(files),
    settings.environment,
    settings.reliabilityPriority,
  );
}

export function decodeProjectV2WavReliable(
  wavBytes: Uint8Array,
): ProjectFileData[] {
  return JSON.parse(
    decode_project_v2_wav_adaptive_wasm(wavBytes),
  ) as ProjectFileData[];
}

export function decodeProjectV2RecordedPcm(
  samples: Float32Array,
  sampleRate: number,
): ProjectFileData[] {
  return JSON.parse(
    decode_project_v2_recorded_pcm_adaptive_wasm(samples, sampleRate),
  ) as ProjectFileData[];
}
