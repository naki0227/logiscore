/* tslint:disable */
/* eslint-disable */

export function analyze_text_v2_recorded_pcm_wasm(samples: Float32Array, sample_rate: number, expected_text: string): string;

/**
 * CRC/FEC付きPCMからv2 Project Payloadを復元する。
 */
export function decode_project_v2_pcm_reliable_wasm(samples: Float32Array): string;

/**
 * clean PCMからv2 Project Payloadを復元する。
 */
export function decode_project_v2_pcm_wasm(samples: Float32Array): string;

export function decode_project_v2_recorded_pcm_adaptive_wasm(samples: Float32Array, sample_rate: number): string;

export function decode_project_v2_recorded_pcm_secure_wasm(samples: Float32Array, sample_rate: number, password: string): string;

export function decode_project_v2_recorded_pcm_wasm(samples: Float32Array, sample_rate: number): string;

/**
 * v2 Project PayloadをDense MIDIから復元する。
 */
export function decode_project_v2_wasm(midi_bytes: Uint8Array): string;

export function decode_project_v2_wav_adaptive_wasm(bytes: Uint8Array): string;

export function decode_project_v2_wav_reliable_wasm(wav_bytes: Uint8Array): string;

export function decode_project_v2_wav_secure_wasm(bytes: Uint8Array, password: string): string;

export function decode_project_wasm(midi_bytes: Uint8Array): any;

/**
 * CRC/FEC付きPCMからv2 Source File Payloadを復元する。
 */
export function decode_source_file_v2_pcm_reliable_wasm(samples: Float32Array): string;

/**
 * clean PCMからv2 Source File Payloadを復元する。
 */
export function decode_source_file_v2_pcm_wasm(samples: Float32Array): string;

export function decode_source_file_v2_recorded_pcm_adaptive_wasm(samples: Float32Array, sample_rate: number): string;

export function decode_source_file_v2_recorded_pcm_secure_wasm(samples: Float32Array, sample_rate: number, password: string): string;

export function decode_source_file_v2_recorded_pcm_wasm(samples: Float32Array, sample_rate: number): string;

/**
 * v2 Source File PayloadをDense MIDIから復元する。
 */
export function decode_source_file_v2_wasm(midi_bytes: Uint8Array): string;

export function decode_source_file_v2_wav_adaptive_wasm(bytes: Uint8Array): string;

export function decode_source_file_v2_wav_reliable_wasm(wav_bytes: Uint8Array): string;

export function decode_source_file_v2_wav_secure_wasm(bytes: Uint8Array, password: string): string;

/**
 * CRC/FEC付きPCMからv2 Text Payloadを復元する。
 */
export function decode_text_v2_pcm_reliable_wasm(samples: Float32Array): string;

/**
 * clean PCMからv2 Text Payloadを復元する。
 */
export function decode_text_v2_pcm_wasm(samples: Float32Array): string;

export function decode_text_v2_recorded_pcm_adaptive_wasm(samples: Float32Array, sample_rate: number): string;

export function decode_text_v2_recorded_pcm_secure_wasm(samples: Float32Array, sample_rate: number, password: string): string;

export function decode_text_v2_recorded_pcm_wasm(samples: Float32Array, sample_rate: number): string;

/**
 * v2 Text PayloadをDense MIDIから復元する。
 */
export function decode_text_v2_wasm(midi_bytes: Uint8Array): string;

export function decode_text_v2_wav_adaptive_wasm(bytes: Uint8Array): string;

export function decode_text_v2_wav_reliable_wasm(wav_bytes: Uint8Array): string;

export function decode_text_v2_wav_secure_wasm(bytes: Uint8Array, password: string): string;

export function decode_v2_recorded_pcm_secure_wasm(samples: Float32Array, sample_rate: number, password: string): string;

export function decode_v2_wav_secure_wasm(bytes: Uint8Array, password: string): string;

/**
 * MIDI バイナリをソースコードにデコードする（WASM 公開 API）。
 * 戻り値は { source: string, extension: string } の Promise/Result。
 */
export function decode_wasm(midi_bytes: Uint8Array): string;

export function describe_acoustic_profile_wasm(environment: string, reliability_priority: number, payload_bytes: number): string;

/**
 * Project Payloadをv2 Musical MIDIでエンコードする。
 */
export function encode_project_v2_musical_wasm(input_json: string): Uint8Array;

/**
 * Project PayloadをCRC/FEC付きPCMへエンコードする。
 */
export function encode_project_v2_pcm_reliable_wasm(input_json: string): Float32Array;

/**
 * Project Payloadをclean PCMへエンコードする。
 */
export function encode_project_v2_pcm_wasm(input_json: string): Float32Array;

/**
 * Project Payloadをv2 Binary Header + Dense MIDIでエンコードする。
 */
export function encode_project_v2_wasm(input_json: string): Uint8Array;

export function encode_project_v2_wav_adaptive_wasm(input_json: string, environment: string, reliability_priority: number): Uint8Array;

export function encode_project_v2_wav_reliable_wasm(input_json: string): Uint8Array;

export function encode_project_v2_wav_secure_wasm(input_json: string, password: string, environment: string, reliability_priority: number): Uint8Array;

/**
 * プロジェクト全体のファイルを MIDI バイナリにエンコードする（WASM 公開 API）。
 * input: JSON string of ProjectFile[]
 */
export function encode_project_wasm(input_json: string): Uint8Array;

/**
 * Source File Payloadをv2 Musical MIDIでエンコードする。
 */
export function encode_source_file_v2_musical_wasm(filename: string, extension: string, source: string): Uint8Array;

/**
 * Source File PayloadをCRC/FEC付きPCMへエンコードする。
 */
export function encode_source_file_v2_pcm_reliable_wasm(filename: string, extension: string, source: string): Float32Array;

/**
 * Source File Payloadをclean PCMへエンコードする。
 */
export function encode_source_file_v2_pcm_wasm(filename: string, extension: string, source: string): Float32Array;

/**
 * Source File Payloadをv2 Binary Header + Dense MIDIでエンコードする。
 */
export function encode_source_file_v2_wasm(filename: string, extension: string, source: string): Uint8Array;

export function encode_source_file_v2_wav_adaptive_wasm(filename: string, extension: string, source: string, environment: string, reliability_priority: number): Uint8Array;

export function encode_source_file_v2_wav_reliable_wasm(filename: string, extension: string, source: string): Uint8Array;

export function encode_source_file_v2_wav_secure_wasm(filename: string, extension: string, source: string, password: string, environment: string, reliability_priority: number): Uint8Array;

/**
 * Text Payloadをv2 Musical MIDIでエンコードする。
 */
export function encode_text_v2_musical_wasm(text: string): Uint8Array;

/**
 * Text PayloadをCRC/FEC付きPCMへエンコードする。
 */
export function encode_text_v2_pcm_reliable_wasm(text: string): Float32Array;

/**
 * Text Payloadをclean PCMへエンコードする。
 */
export function encode_text_v2_pcm_wasm(text: string): Float32Array;

/**
 * Text Payloadをv2 Binary Header + Dense MIDIでエンコードする。
 */
export function encode_text_v2_wasm(text: string): Uint8Array;

export function encode_text_v2_wav_adaptive_wasm(text: string, environment: string, reliability_priority: number): Uint8Array;

export function encode_text_v2_wav_reliable_wasm(text: string): Uint8Array;

export function encode_text_v2_wav_secure_wasm(text: string, password: string, environment: string, reliability_priority: number): Uint8Array;

/**
 * ソースコードを MIDI バイナリにエンコードする（WASM 公開 API）。
 */
export function encode_wasm(source: string, extension: string): Uint8Array;

/**
 * 拡張子のメタ情報を JSON 文字列で返す（WASM 公開 API）。
 */
export function get_extension_info(extension: string): string;

export function get_pcm_sample_rate(): number;

/**
 * v2 packet protocolのバージョンを返す。
 */
export function get_v2_version(): number;

/**
 * 現在のプロトコルバージョンを返す
 */
export function get_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly decode_project_v2_recorded_pcm_wasm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly decode_project_v2_wav_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_project_wasm: (a: number, b: number) => [number, number, number];
    readonly decode_source_file_v2_recorded_pcm_wasm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly decode_source_file_v2_wav_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_text_v2_recorded_pcm_wasm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly decode_text_v2_wav_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_v2_wav_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_source_file_v2_wav_reliable_wasm: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly encode_text_v2_wav_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly get_extension_info: (a: number, b: number) => [number, number];
    readonly get_version: () => [number, number];
    readonly decode_project_v2_recorded_pcm_secure_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly decode_project_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly decode_source_file_v2_recorded_pcm_secure_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly decode_source_file_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly decode_text_v2_recorded_pcm_secure_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly decode_text_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly decode_v2_recorded_pcm_secure_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly decode_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly encode_project_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly encode_source_file_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number, number];
    readonly encode_text_v2_wav_secure_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly analyze_text_v2_recorded_pcm_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly decode_project_v2_recorded_pcm_adaptive_wasm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly decode_project_v2_wav_adaptive_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_source_file_v2_recorded_pcm_adaptive_wasm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly decode_source_file_v2_wav_adaptive_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_text_v2_recorded_pcm_adaptive_wasm: (a: number, b: number, c: number) => [number, number, number, number];
    readonly decode_text_v2_wav_adaptive_wasm: (a: number, b: number) => [number, number, number, number];
    readonly describe_acoustic_profile_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly encode_project_v2_wav_adaptive_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly encode_source_file_v2_wav_adaptive_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly encode_text_v2_wav_adaptive_wasm: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly decode_project_v2_pcm_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_project_v2_pcm_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_project_v2_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_source_file_v2_pcm_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_source_file_v2_pcm_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_source_file_v2_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_text_v2_pcm_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_text_v2_pcm_wasm: (a: number, b: number) => [number, number, number, number];
    readonly decode_text_v2_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_v2_musical_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_v2_pcm_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_v2_pcm_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_v2_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_source_file_v2_musical_wasm: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly encode_source_file_v2_pcm_reliable_wasm: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly encode_source_file_v2_pcm_wasm: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly encode_source_file_v2_wasm: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly encode_text_v2_musical_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_text_v2_pcm_reliable_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_text_v2_pcm_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_text_v2_wasm: (a: number, b: number) => [number, number, number, number];
    readonly get_pcm_sample_rate: () => number;
    readonly get_v2_version: () => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
