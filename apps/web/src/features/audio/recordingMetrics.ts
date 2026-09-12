import { analyzeTextV2RecordedPcm } from "../../lib/acoustic-codec";
import { decodeAudioFile } from "./decodeAudioFile";

export interface RecordingMeasurement {
  profileId: number | null;
  finalRecoveryRatePercent: number;
  rawSymbolAccuracyPercent: number;
  correctedErrors: number;
  payloadBitrateBps: number;
  decodeTimeMicros: number;
  playbackDurationMs: number;
}

export async function measureTextRecordingFile(
  file: File,
  expectedText: string,
): Promise<RecordingMeasurement> {
  const audio = await decodeAudioFile(file);
  const startedAt = performance.now();
  const telemetry = analyzeTextV2RecordedPcm(
    audio.samples,
    audio.sampleRate,
    expectedText,
  );
  const decodeTimeMicros = (performance.now() - startedAt) * 1_000;
  const playbackDurationMs = (audio.samples.length * 1_000) / audio.sampleRate;
  const payloadBits = new TextEncoder().encode(expectedText).length * 8;
  return {
    ...telemetry,
    payloadBitrateBps: (payloadBits * 1_000) / playbackDurationMs,
    decodeTimeMicros,
    playbackDurationMs,
  };
}
