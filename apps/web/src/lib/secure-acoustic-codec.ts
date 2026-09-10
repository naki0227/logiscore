import {
  decode_project_v2_recorded_pcm_secure_wasm,
  decode_project_v2_wav_secure_wasm,
  decode_source_file_v2_recorded_pcm_secure_wasm,
  decode_source_file_v2_wav_secure_wasm,
  decode_text_v2_recorded_pcm_secure_wasm,
  decode_text_v2_wav_secure_wasm,
  decode_v2_recorded_pcm_secure_wasm,
  decode_v2_wav_secure_wasm,
  encode_project_v2_wav_secure_wasm,
  encode_source_file_v2_wav_secure_wasm,
  encode_text_v2_wav_secure_wasm,
} from "../pkg/harmonic_core.js";
import type { AcousticSettings } from "../features/acoustic/model";
import type { ProjectFileData, SourceFileV2 } from "./wasm-loader";

export type SecureDecodedPayload =
  | { type: "text"; text: string }
  | ({ type: "source-file"; version: 2 } & SourceFileV2)
  | { type: "project"; version: 2; files: ProjectFileData[] };

export function encodeTextV2WavSecure(
  text: string,
  password: string,
  settings: AcousticSettings,
): Uint8Array {
  return encode_text_v2_wav_secure_wasm(
    text,
    password,
    settings.environment,
    settings.reliabilityPriority,
  );
}

export function decodeTextV2WavSecure(
  wavBytes: Uint8Array,
  password: string,
): string {
  return decode_text_v2_wav_secure_wasm(wavBytes, password);
}

export function decodeTextV2RecordedPcmSecure(
  samples: Float32Array,
  sampleRate: number,
  password: string,
): string {
  return decode_text_v2_recorded_pcm_secure_wasm(samples, sampleRate, password);
}

export function encodeSourceFileV2WavSecure(
  filename: string,
  extension: string,
  source: string,
  password: string,
  settings: AcousticSettings,
): Uint8Array {
  return encode_source_file_v2_wav_secure_wasm(
    filename,
    extension,
    source,
    password,
    settings.environment,
    settings.reliabilityPriority,
  );
}

export function decodeSourceFileV2WavSecure(
  wavBytes: Uint8Array,
  password: string,
): SourceFileV2 {
  return JSON.parse(
    decode_source_file_v2_wav_secure_wasm(wavBytes, password),
  ) as SourceFileV2;
}

export function decodeSourceFileV2RecordedPcmSecure(
  samples: Float32Array,
  sampleRate: number,
  password: string,
): SourceFileV2 {
  return JSON.parse(
    decode_source_file_v2_recorded_pcm_secure_wasm(
      samples,
      sampleRate,
      password,
    ),
  ) as SourceFileV2;
}

export function encodeProjectV2WavSecure(
  files: ProjectFileData[],
  password: string,
  settings: AcousticSettings,
): Uint8Array {
  return encode_project_v2_wav_secure_wasm(
    JSON.stringify(files),
    password,
    settings.environment,
    settings.reliabilityPriority,
  );
}

export function decodeProjectV2WavSecure(
  wavBytes: Uint8Array,
  password: string,
): ProjectFileData[] {
  return JSON.parse(
    decode_project_v2_wav_secure_wasm(wavBytes, password),
  ) as ProjectFileData[];
}

export function decodeProjectV2RecordedPcmSecure(
  samples: Float32Array,
  sampleRate: number,
  password: string,
): ProjectFileData[] {
  return JSON.parse(
    decode_project_v2_recorded_pcm_secure_wasm(samples, sampleRate, password),
  ) as ProjectFileData[];
}

export function decodeV2WavSecure(
  wavBytes: Uint8Array,
  password: string,
): SecureDecodedPayload {
  return parseSecurePayload(decode_v2_wav_secure_wasm(wavBytes, password));
}

export function decodeV2RecordedPcmSecure(
  samples: Float32Array,
  sampleRate: number,
  password: string,
): SecureDecodedPayload {
  return parseSecurePayload(
    decode_v2_recorded_pcm_secure_wasm(samples, sampleRate, password),
  );
}

export function parseSecurePayload(json: string): SecureDecodedPayload {
  const value: unknown = JSON.parse(json);
  if (typeof value !== "object" || value === null || !("type" in value)) {
    throw new Error("Invalid secure payload response");
  }
  if (
    value.type === "text" &&
    "text" in value &&
    typeof value.text === "string"
  ) {
    return { type: "text", text: value.text };
  }
  if (isSourceFile(value)) {
    return {
      type: "source-file",
      version: 2,
      filename: value.filename,
      extension: value.extension,
      source: value.source,
    };
  }
  if (
    value.type === "project" &&
    "version" in value &&
    value.version === 2 &&
    "files" in value &&
    Array.isArray(value.files) &&
    value.files.every(isProjectFile)
  ) {
    return { type: "project", version: 2, files: value.files };
  }
  throw new Error("Invalid secure payload response");
}

function isSourceFile(
  value: object & Record<"type", unknown>,
): value is typeof value & SourceFileV2 & { type: "source-file"; version: 2 } {
  return (
    value.type === "source-file" &&
    "version" in value &&
    value.version === 2 &&
    "filename" in value &&
    typeof value.filename === "string" &&
    "extension" in value &&
    typeof value.extension === "string" &&
    "source" in value &&
    typeof value.source === "string"
  );
}

function isProjectFile(value: unknown): value is ProjectFileData {
  return (
    typeof value === "object" &&
    value !== null &&
    "name" in value &&
    typeof value.name === "string" &&
    "extension" in value &&
    typeof value.extension === "string" &&
    "source" in value &&
    typeof value.source === "string"
  );
}
