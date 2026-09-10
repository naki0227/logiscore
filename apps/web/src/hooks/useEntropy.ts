import { useState, useCallback, useRef } from "react";
import { detectImportedPayload } from "../features/files/detectImportedPayload";
import {
  initWasm,
  decode,
  decodeProject as decodeProjectV1,
  decodeProjectV2,
  decodeSourceFileV2,
  decodeTextV2,
  encodeSourceFileV2,
  encodeSourceFileV2Musical,
  encodeTextV2,
  encodeTextV2Musical,
  encodeProjectV2,
  encodeProjectV2Musical,
  getExtensionInfo,
  getV2Version,
  getVersion,
} from "../lib/wasm-loader";

export interface EntropyState {
  /** WASM 初期化済みか */
  ready: boolean;
  /** 処理中か */
  processing: boolean;
  /** エンコード済みの MIDI バイナリ */
  midiData: Uint8Array | null;
  /** デコード済みのソースコード */
  decodedSource: string | null;
  /** 拡張子情報 */
  extensionInfo: {
    scale_id: number;
    root_key: number;
    name: string;
    scale_name: string;
  } | null;
  /** エラーメッセージ */
  error: string | null;
  /** プロトコルバージョン */
  systemVersion: string | null;
  /** v2 packet protocol version */
  v2Version: number | null;
}

export interface ProjectFile {
  name: string;
  source: string;
  extension: string;
}

export type ImportedPayload =
  | { type: "project"; version: 1 | 2; files: ProjectFile[] }
  | { type: "text"; text: string }
  | {
      type: "source-file";
      version: 1 | 2;
      filename?: string;
      source: string;
      extension: string;
    };

export function useEntropy() {
  const [state, setState] = useState<EntropyState>({
    ready: false,
    processing: false,
    midiData: null,
    decodedSource: null,
    extensionInfo: null,
    error: null,
    systemVersion: null,
    v2Version: null,
  });

  const readyRef = useRef(false);

  /** WASM を初期化 */
  const initialize = useCallback(async () => {
    try {
      await initWasm();
      const version = getVersion();
      const v2Version = getV2Version();
      readyRef.current = true;
      setState((prev) => ({
        ...prev,
        ready: true,
        systemVersion: version,
        v2Version,
      }));
    } catch (e) {
      setState((prev) => ({ ...prev, error: `WASM init failed: ${e}` }));
    }
  }, []);

  /** プロジェクト全体のエンコード */
  const encodeProjectSource = useCallback(
    (files: ProjectFile[], musical = false) => {
      if (!readyRef.current) return null;
      setState((prev) => ({ ...prev, processing: true, error: null }));
      try {
        // 便宜上最初のファイルの拡張子情報を代表としてセット
        const firstExt = files[0]?.extension || ".rs";
        const info = getExtensionInfo(firstExt);

        const midi = musical
          ? encodeProjectV2Musical(files)
          : encodeProjectV2(files);

        setState((prev) => ({
          ...prev,
          processing: false,
          midiData: midi,
          extensionInfo: info,
        }));
        return midi;
      } catch (e) {
        setState((prev) => ({
          ...prev,
          processing: false,
          error: `Project encode failed: ${e}`,
        }));
        return null;
      }
    },
    [],
  );

  const encodeText = useCallback((text: string, musical = false) => {
    if (!readyRef.current) return null;
    setState((prev) => ({ ...prev, processing: true, error: null }));
    try {
      const midi = musical ? encodeTextV2Musical(text) : encodeTextV2(text);
      setState((prev) => ({
        ...prev,
        processing: false,
        midiData: midi,
        extensionInfo: null,
      }));
      return midi;
    } catch (cause) {
      setState((prev) => ({
        ...prev,
        processing: false,
        error: `Text encode failed: ${cause}`,
      }));
      return null;
    }
  }, []);

  const encodeSourceFile = useCallback(
    (filename: string, extension: string, source: string, musical = false) => {
      if (!readyRef.current) return null;
      setState((prev) => ({ ...prev, processing: true, error: null }));
      try {
        const midi = musical
          ? encodeSourceFileV2Musical(filename, extension, source)
          : encodeSourceFileV2(filename, extension, source);
        const info = getExtensionInfo(extension);
        setState((prev) => ({
          ...prev,
          processing: false,
          midiData: midi,
          extensionInfo: info,
        }));
        return midi;
      } catch (cause) {
        setState((prev) => ({
          ...prev,
          processing: false,
          error: `Source File encode failed: ${cause}`,
        }));
        return null;
      }
    },
    [],
  );

  /** プロジェクト全体のデコード */
  const decodeProjectSource = useCallback((midiBytes: Uint8Array) => {
    if (!readyRef.current) return null;
    setState((prev) => ({ ...prev, processing: true, error: null }));
    try {
      const files = decodeProjectV2(midiBytes);
      setState((prev) => ({
        ...prev,
        processing: false,
      }));
      return files;
    } catch (e) {
      setState((prev) => ({
        ...prev,
        processing: false,
      }));
      throw e;
    }
  }, []);

  const decodeText = useCallback((midiBytes: Uint8Array) => {
    if (!readyRef.current) return null;
    setState((prev) => ({ ...prev, processing: true, error: null }));
    try {
      const text = decodeTextV2(midiBytes);
      setState((prev) => ({
        ...prev,
        processing: false,
        decodedSource: text,
        midiData: midiBytes,
      }));
      return text;
    } catch (cause) {
      setState((prev) => ({
        ...prev,
        processing: false,
        error: `Text decode failed: ${cause}`,
      }));
      throw cause;
    }
  }, []);

  const decodeSourceFile = useCallback((midiBytes: Uint8Array) => {
    if (!readyRef.current) return null;
    setState((prev) => ({ ...prev, processing: true, error: null }));
    try {
      const file = decodeSourceFileV2(midiBytes);
      setState((prev) => ({
        ...prev,
        processing: false,
        decodedSource: file.source,
        midiData: midiBytes,
      }));
      return file;
    } catch (cause) {
      setState((prev) => ({
        ...prev,
        processing: false,
        error: `Source File decode failed: ${cause}`,
      }));
      throw cause;
    }
  }, []);

  const decodeImportedMidi = useCallback(
    (midiBytes: Uint8Array): ImportedPayload => {
      const result = detectImportedPayload(midiBytes, {
        projectV2: decodeProjectV2,
        projectV1: decodeProjectV1,
        text: decodeTextV2,
        sourceV2: decodeSourceFileV2,
        sourceV1: decode,
      });

      setState((prev) => ({
        ...prev,
        midiData: midiBytes,
        decodedSource:
          result.type === "text"
            ? result.text
            : result.type === "source-file"
              ? result.source
              : null,
        error: null,
      }));

      return result;
    },
    [],
  );

  return {
    ...state,
    initialize,
    encodeProjectSource,
    encodeText,
    encodeSourceFile,
    decodeProjectSource,
    decodeText,
    decodeSourceFile,
    decodeImportedMidi,
  };
}
