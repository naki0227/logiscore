/* tslint:disable */
/* eslint-disable */

export function decode_project_wasm(midi_bytes: Uint8Array): any;

/**
 * MIDI バイナリをソースコードにデコードする（WASM 公開 API）。
 * 戻り値は { source: string, extension: string } の Promise/Result。
 */
export function decode_wasm(midi_bytes: Uint8Array): string;

/**
 * プロジェクト全体のファイルを MIDI バイナリにエンコードする（WASM 公開 API）。
 * input: JSON string of ProjectFile[]
 */
export function encode_project_wasm(input_json: string): Uint8Array;

/**
 * ソースコードを MIDI バイナリにエンコードする（WASM 公開 API）。
 */
export function encode_wasm(source: string, extension: string): Uint8Array;

/**
 * 拡張子のメタ情報を JSON 文字列で返す（WASM 公開 API）。
 */
export function get_extension_info(extension: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly decode_project_wasm: (a: number, b: number) => [number, number, number];
    readonly decode_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_project_wasm: (a: number, b: number) => [number, number, number, number];
    readonly encode_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly get_extension_info: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
