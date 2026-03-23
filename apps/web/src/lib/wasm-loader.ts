import init, {
  encode_wasm,
  encode_project_wasm,
  decode_wasm,
  decode_project_wasm,
  get_extension_info,
  get_version as get_version_wasm,
} from '../pkg/harmonic_core.js';

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
export function decode(midiBytes: Uint8Array): { source: string, extension: string } {
  return JSON.parse(decode_wasm(midiBytes));
}

/**
 * プロジェクト全体を MIDI バイナリにエンコードする。
 */
export function encodeProject(files: { name: string, source: string, extension: string }[]): Uint8Array {
  return encode_project_wasm(JSON.stringify(files));
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
export function decodeProject(midiBytes: Uint8Array): { name: string, source: string, extension: string }[] {
  return JSON.parse(decode_project_wasm(midiBytes));
}

/**
 * 現在のプロトコルバージョンを取得する。
 */
export function getVersion(): string {
  return get_version_wasm();
}
