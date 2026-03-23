import { useState, useCallback, useRef } from 'react';
import { initWasm, encode, decode, encodeProject, decodeProject, getExtensionInfo } from '../lib/wasm-loader';

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
  extensionInfo: { scale_id: number; root_key: number; name: string; scale_name: string } | null;
  /** エラーメッセージ */
  error: string | null;
}

export interface ProjectFile {
    name: string;
    source: string;
    extension: string;
}

export function useEntropy() {
  const [state, setState] = useState<EntropyState>({
    ready: false,
    processing: false,
    midiData: null,
    decodedSource: null,
    extensionInfo: null,
    error: null,
  });

  const readyRef = useRef(false);

  /** WASM を初期化 */
  const initialize = useCallback(async () => {
    try {
      await initWasm();
      readyRef.current = true;
      setState(prev => ({ ...prev, ready: true }));
    } catch (e) {
      setState(prev => ({ ...prev, error: `WASM init failed: ${e}` }));
    }
  }, []);

  /** ソースコードをエンコード */
  const encodeSource = useCallback((source: string, extension: string) => {
    if (!readyRef.current) return null;
    setState(prev => ({ ...prev, processing: true, error: null }));
    try {
      const info = getExtensionInfo(extension);
      const midi = encode(source, extension);
      setState(prev => ({
        ...prev,
        processing: false,
        midiData: midi,
        extensionInfo: info,
      }));
      return midi;
    } catch (e) {
      setState(prev => ({
        ...prev,
        processing: false,
        error: `Encode failed: ${e}`,
      }));
      return null;
    }
  }, []);

  /** プロジェクト全体のエンコード */
  const encodeProjectSource = useCallback((files: ProjectFile[]) => {
    if (!readyRef.current) return null;
    setState(prev => ({ ...prev, processing: true, error: null }));
    try {
      // 便宜上最初のファイルの拡張子情報を代表としてセット
      const firstExt = files[0]?.extension || '.rs';
      const info = getExtensionInfo(firstExt);
      
      const midi = encodeProject(files);
      
      setState(prev => ({
        ...prev,
        processing: false,
        midiData: midi,
        extensionInfo: info,
      }));
      return midi;
    } catch (e) {
      setState(prev => ({
        ...prev,
        processing: false,
        error: `Project encode failed: ${e}`,
      }));
      return null;
    }
  }, []);

  /** MIDI バイナリをデコード */
  const decodeSource = useCallback((midiBytes: Uint8Array) => {
    if (!readyRef.current) return null;
    setState(prev => ({ ...prev, processing: true, error: null }));
    try {
      const result = decode(midiBytes);
      setState(prev => ({
        ...prev,
        processing: false,
        decodedSource: result.source,
        midiData: midiBytes, 
      }));
      return result;
    } catch (e) {
      setState(prev => ({
        ...prev,
        processing: false,
      }));
      throw e;
    }
  }, []);

  /** プロジェクト全体のデコード */
  const decodeProjectSource = useCallback((midiBytes: Uint8Array) => {
    if (!readyRef.current) return null;
    setState(prev => ({ ...prev, processing: true, error: null }));
    try {
        const files = decodeProject(midiBytes);
        setState(prev => ({
            ...prev,
            processing: false,
        }));
        return files;
    } catch (e) {
        setState(prev => ({
            ...prev,
            processing: false,
        }));
        throw e;
    }
  }, []);

  return {
    ...state,
    initialize,
    encodeSource,
    encodeProjectSource,
    decodeSource,
    decodeProjectSource,
  };
}
