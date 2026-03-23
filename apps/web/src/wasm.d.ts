declare module '../../../../packages/harmonic-core/pkg/harmonic_core.js' {
  export default function init(): Promise<void>;
  export function encode_wasm(source: string, extension: string): Uint8Array;
  export function decode_wasm(midi_bytes: Uint8Array): string;
  export function get_extension_info(extension: string): string;
}
