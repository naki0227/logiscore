import init, {
  encode_wasm,
  encode_project_wasm,
  encode_project_v2_wasm,
  encode_project_v2_musical_wasm,
  encode_project_v2_pcm_wasm,
  encode_project_v2_pcm_reliable_wasm,
  decode_wasm,
  decode_project_wasm,
  decode_project_v2_wasm,
  decode_project_v2_pcm_wasm,
  decode_project_v2_pcm_reliable_wasm,
  decode_source_file_v2_wasm,
  decode_source_file_v2_pcm_wasm,
  decode_source_file_v2_pcm_reliable_wasm,
  decode_text_v2_wasm,
  decode_text_v2_pcm_wasm,
  decode_text_v2_pcm_reliable_wasm,
  encode_source_file_v2_wasm,
  encode_source_file_v2_musical_wasm,
  encode_source_file_v2_pcm_wasm,
  encode_source_file_v2_pcm_reliable_wasm,
  encode_text_v2_wasm,
  encode_text_v2_musical_wasm,
  encode_text_v2_pcm_wasm,
  encode_text_v2_pcm_reliable_wasm,
  get_extension_info,
  get_pcm_sample_rate,
  get_v2_version,
  get_version as get_version_wasm,
} from "../pkg/harmonic_core.js";

let initialized = false;

/**
 * WASM モジュールを初期化する（一度だけ）。
 */
export async function initWasm(): Promise<void> {
  if (initialized) return;
  await init();
  initialized = true;
}

/**
 * ソースコードを MIDI バイナリにエンコードする。
 */
export function encode(source: string, extension: string): Uint8Array {
  return encode_wasm(source, extension);
}

/**
 * MIDI バイナリをソースコードにデコードする。
 */
export function decode(midiBytes: Uint8Array): {
  source: string;
  extension: string;
} {
  return JSON.parse(decode_wasm(midiBytes));
}

export function encodeTextV2(text: string): Uint8Array {
  return encode_text_v2_wasm(text);
}

export function encodeTextV2Musical(text: string): Uint8Array {
  return encode_text_v2_musical_wasm(text);
}

export function decodeTextV2(midiBytes: Uint8Array): string {
  return decode_text_v2_wasm(midiBytes);
}

export function encodeTextV2Pcm(text: string): Float32Array {
  return encode_text_v2_pcm_wasm(text);
}

export function decodeTextV2Pcm(samples: Float32Array): string {
  return decode_text_v2_pcm_wasm(samples);
}

export function encodeTextV2PcmReliable(text: string): Float32Array {
  return encode_text_v2_pcm_reliable_wasm(text);
}

export function decodeTextV2PcmReliable(samples: Float32Array): string {
  return decode_text_v2_pcm_reliable_wasm(samples);
}

export interface SourceFileV2 {
  filename: string;
  extension: string;
  source: string;
}

export function encodeSourceFileV2(
  filename: string,
  extension: string,
  source: string,
): Uint8Array {
  return encode_source_file_v2_wasm(filename, extension, source);
}

export function encodeSourceFileV2Musical(
  filename: string,
  extension: string,
  source: string,
): Uint8Array {
  return encode_source_file_v2_musical_wasm(filename, extension, source);
}

export function decodeSourceFileV2(midiBytes: Uint8Array): SourceFileV2 {
  return JSON.parse(decode_source_file_v2_wasm(midiBytes)) as SourceFileV2;
}

export function encodeSourceFileV2Pcm(
  filename: string,
  extension: string,
  source: string,
): Float32Array {
  return encode_source_file_v2_pcm_wasm(filename, extension, source);
}

export function decodeSourceFileV2Pcm(samples: Float32Array): SourceFileV2 {
  return JSON.parse(decode_source_file_v2_pcm_wasm(samples)) as SourceFileV2;
}

export function encodeSourceFileV2PcmReliable(
  filename: string,
  extension: string,
  source: string,
): Float32Array {
  return encode_source_file_v2_pcm_reliable_wasm(filename, extension, source);
}

export function decodeSourceFileV2PcmReliable(
  samples: Float32Array,
): SourceFileV2 {
  return JSON.parse(
    decode_source_file_v2_pcm_reliable_wasm(samples),
  ) as SourceFileV2;
}

export function getV2Version(): number {
  return get_v2_version();
}

export function getPcmSampleRate(): number {
  return get_pcm_sample_rate();
}

/**
 * プロジェクト全体を MIDI バイナリにエンコードする。
 */
export function encodeProject(
  files: { name: string; source: string; extension: string }[],
): Uint8Array {
  return encode_project_wasm(JSON.stringify(files));
}

export interface ProjectFileData {
  name: string;
  source: string;
  extension: string;
}

export function encodeProjectV2(files: ProjectFileData[]): Uint8Array {
  return encode_project_v2_wasm(JSON.stringify(files));
}

export function encodeProjectV2Musical(files: ProjectFileData[]): Uint8Array {
  return encode_project_v2_musical_wasm(JSON.stringify(files));
}

export function decodeProjectV2(midiBytes: Uint8Array): ProjectFileData[] {
  return JSON.parse(decode_project_v2_wasm(midiBytes)) as ProjectFileData[];
}

export function encodeProjectV2Pcm(files: ProjectFileData[]): Float32Array {
  return encode_project_v2_pcm_wasm(JSON.stringify(files));
}

export function decodeProjectV2Pcm(samples: Float32Array): ProjectFileData[] {
  return JSON.parse(decode_project_v2_pcm_wasm(samples)) as ProjectFileData[];
}

export function encodeProjectV2PcmReliable(
  files: ProjectFileData[],
): Float32Array {
  return encode_project_v2_pcm_reliable_wasm(JSON.stringify(files));
}

export function decodeProjectV2PcmReliable(
  samples: Float32Array,
): ProjectFileData[] {
  return JSON.parse(
    decode_project_v2_pcm_reliable_wasm(samples),
  ) as ProjectFileData[];
}

/**
 * 拡張子のメタ情報を取得する。
 */
export function getExtensionInfo(extension: string): {
  scale_id: number;
  root_key: number;
  name: string;
  scale_name: string;
} {
  return JSON.parse(get_extension_info(extension));
}

/**
 * プロジェクト全体の MIDI バイナリをデコードする。
 */
export function decodeProject(
  midiBytes: Uint8Array,
): { name: string; source: string; extension: string }[] {
  return JSON.parse(decode_project_wasm(midiBytes));
}

/**
 * 現在のプロトコルバージョンを取得する。
 */
export function getVersion(): string {
  return get_version_wasm();
}
