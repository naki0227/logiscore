import { useCallback, useState } from "react";
import {
  decodeProjectV2RecordedPcm,
  decodeProjectV2WavReliable,
  decodeSourceFileV2RecordedPcm,
  decodeSourceFileV2WavReliable,
  decodeTextV2RecordedPcm,
  decodeTextV2WavReliable,
  describeAcousticProfile,
  encodeProjectV2WavReliable,
  encodeSourceFileV2WavReliable,
  encodeTextV2WavReliable,
} from "../lib/acoustic-codec";
import {
  decodeV2RecordedPcmSecure,
  decodeV2WavSecure,
  encodeProjectV2WavSecure,
  encodeSourceFileV2WavSecure,
  encodeTextV2WavSecure,
} from "../lib/secure-acoustic-codec";
import { decodeAudioFile } from "../features/audio/decodeAudioFile";
import type { AcousticSettings } from "../features/acoustic/model";
import type { ImportedPayload, ProjectFile } from "./useEntropy";

export function useAcousticCodec() {
  const [wavData, setWavData] = useState<Uint8Array | null>(null);
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const encodeAndStore = useCallback((encode: () => Uint8Array) => {
    setProcessing(true);
    setError(null);
    try {
      const wav = encode();
      setWavData(wav);
      return wav;
    } catch {
      setError("Audio encode failed");
      return null;
    } finally {
      setProcessing(false);
    }
  }, []);

  const encodeText = useCallback(
    (text: string, settings: AcousticSettings, password?: string) => {
      return encodeAndStore(() =>
        password === undefined
          ? encodeTextV2WavReliable(text, settings)
          : encodeTextV2WavSecure(text, password, settings),
      );
    },
    [encodeAndStore],
  );

  const encodeSourceFile = useCallback(
    (
      filename: string,
      extension: string,
      source: string,
      settings: AcousticSettings,
      password?: string,
    ) =>
      encodeAndStore(() =>
        password === undefined
          ? encodeSourceFileV2WavReliable(filename, extension, source, settings)
          : encodeSourceFileV2WavSecure(
              filename,
              extension,
              source,
              password,
              settings,
            ),
      ),
    [encodeAndStore],
  );

  const encodeProject = useCallback(
    (files: ProjectFile[], settings: AcousticSettings, password?: string) =>
      encodeAndStore(() =>
        password === undefined
          ? encodeProjectV2WavReliable(files, settings)
          : encodeProjectV2WavSecure(files, password, settings),
      ),
    [encodeAndStore],
  );

  const decodeWav = useCallback(
    (bytes: Uint8Array, password?: string): ImportedPayload =>
      detectWavPayload(bytes, password),
    [],
  );

  const decodeFile = useCallback(async (file: File, password?: string) => {
    setProcessing(true);
    setError(null);
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      let decoded: ImportedPayload;
      if (isWav(file)) {
        decoded = detectWavPayload(bytes, password);
        setWavData(bytes);
      } else {
        const audio = await decodeAudioFile(file);
        decoded = detectRecordedPcm(audio.samples, audio.sampleRate, password);
        setWavData(null);
      }
      return decoded;
    } catch {
      setError("Audio decode failed");
      throw new Error("Audio decode failed");
    } finally {
      setProcessing(false);
    }
  }, []);

  return {
    wavData,
    processing,
    error,
    encodeText,
    encodeSourceFile,
    encodeProject,
    decodeWav,
    decodeFile,
    describeProfile: describeAcousticProfile,
  };
}

function detectWavPayload(
  bytes: Uint8Array,
  password?: string,
): ImportedPayload {
  if (password !== undefined) return detectSecureWavPayload(bytes, password);
  try {
    const files = decodeProjectV2WavReliable(bytes);
    if (files.length > 0) return { type: "project", version: 2, files };
  } catch {
    // Continue through the payload types encoded in the authenticated header.
  }
  try {
    return { type: "text", text: decodeTextV2WavReliable(bytes) };
  } catch {
    const file = decodeSourceFileV2WavReliable(bytes);
    return { type: "source-file", version: 2, ...file };
  }
}

function detectRecordedPcm(
  samples: Float32Array,
  sampleRate: number,
  password?: string,
): ImportedPayload {
  if (password !== undefined) {
    return detectSecureRecordedPcm(samples, sampleRate, password);
  }
  try {
    const files = decodeProjectV2RecordedPcm(samples, sampleRate);
    if (files.length > 0) return { type: "project", version: 2, files };
  } catch {
    // Continue through the payload types encoded in the authenticated header.
  }
  try {
    return { type: "text", text: decodeTextV2RecordedPcm(samples, sampleRate) };
  } catch {
    const file = decodeSourceFileV2RecordedPcm(samples, sampleRate);
    return { type: "source-file", version: 2, ...file };
  }
}

function detectSecureWavPayload(
  bytes: Uint8Array,
  password: string,
): ImportedPayload {
  return decodeV2WavSecure(bytes, password);
}

function detectSecureRecordedPcm(
  samples: Float32Array,
  sampleRate: number,
  password: string,
): ImportedPayload {
  return decodeV2RecordedPcmSecure(samples, sampleRate, password);
}

function isWav(file: File): boolean {
  return file.name.toLowerCase().endsWith(".wav") || file.type === "audio/wav";
}
