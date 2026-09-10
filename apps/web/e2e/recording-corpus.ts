import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export interface RecordingCorpus {
  schemaVersion: 1;
  entries: RecordingEntry[];
}

export interface RecordingEntry {
  id: string;
  status: "planned" | "captured" | "verified";
  text: string;
  sha256: string;
  speaker: string;
  distanceM: number;
  recorderApp: string;
  environment: string;
  deviceModel: string | null;
  recordedAt: string | null;
  recordingFile: string | null;
  measurement: RecordingMeasurement | null;
}

export interface RecordingMeasurement {
  finalRecoveryRatePercent: number;
  rawSymbolAccuracyPercent: number;
  correctedErrors: number;
  payloadBitrateBps: number;
  decodeTimeMicros: number;
  playbackDurationMs: number;
}

export const corpusRoot = path.join(
  import.meta.dirname,
  "fixtures",
  "recordings",
);

export function loadRecordingCorpus(): RecordingCorpus {
  const input: unknown = JSON.parse(
    fs.readFileSync(path.join(corpusRoot, "corpus.json"), "utf8"),
  );
  return parseRecordingCorpus(input);
}

export function parseRecordingCorpus(input: unknown): RecordingCorpus {
  const root = record(input, "corpus");
  if (root.schema_version !== 1 || !Array.isArray(root.entries)) {
    throw new Error("Unsupported recording corpus schema");
  }
  const entries = root.entries.map(parseEntry);
  if (new Set(entries.map((entry) => entry.id)).size !== entries.length) {
    throw new Error("Recording corpus IDs must be unique");
  }
  return { schemaVersion: 1, entries };
}

function parseEntry(input: unknown): RecordingEntry {
  const entry = record(input, "entry");
  const payload = record(entry.payload, "payload");
  const capture = record(entry.capture, "capture");
  const status = oneOf(
    entry.status,
    ["planned", "captured", "verified"] as const,
    "status",
  );
  const text = string(payload.text, "payload.text");
  const parsed: RecordingEntry = {
    id: string(entry.id, "id"),
    status,
    text,
    sha256: string(payload.sha256, "payload.sha256"),
    speaker: string(capture.speaker, "capture.speaker"),
    distanceM: positiveNumber(capture.distance_m, "capture.distance_m"),
    recorderApp: string(capture.recorder_app, "capture.recorder_app"),
    environment: string(capture.environment, "capture.environment"),
    deviceModel: nullableString(capture.device_model, "capture.device_model"),
    recordedAt: nullableString(capture.recorded_at, "capture.recorded_at"),
    recordingFile: nullableString(entry.recording_file, "recording_file"),
    measurement: parseMeasurement(entry.measurement),
  };
  if (payload.type !== "text") throw new Error("Only Text corpus is supported");
  if (createHash("sha256").update(text).digest("hex") !== parsed.sha256) {
    throw new Error(`Payload hash mismatch for ${parsed.id}`);
  }
  if (
    status !== "planned" &&
    (!parsed.deviceModel || !parsed.recordedAt || !parsed.recordingFile)
  ) {
    throw new Error(`Captured corpus metadata is incomplete for ${parsed.id}`);
  }
  if (status === "verified" && !parsed.measurement) {
    throw new Error(`Verified corpus measurement is missing for ${parsed.id}`);
  }
  return parsed;
}

function parseMeasurement(input: unknown): RecordingMeasurement | null {
  if (input === null) return null;
  const value = record(input, "measurement");
  return {
    finalRecoveryRatePercent: percent(
      value.final_recovery_rate_percent,
      "measurement.final_recovery_rate_percent",
    ),
    rawSymbolAccuracyPercent: percent(
      value.raw_symbol_accuracy_percent,
      "measurement.raw_symbol_accuracy_percent",
    ),
    correctedErrors: nonNegativeNumber(
      value.corrected_errors,
      "measurement.corrected_errors",
    ),
    payloadBitrateBps: nonNegativeNumber(
      value.payload_bitrate_bps,
      "measurement.payload_bitrate_bps",
    ),
    decodeTimeMicros: nonNegativeNumber(
      value.decode_time_micros,
      "measurement.decode_time_micros",
    ),
    playbackDurationMs: positiveNumber(
      value.playback_duration_ms,
      "measurement.playback_duration_ms",
    ),
  };
}

function record(value: unknown, name: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
  return value as Record<string, unknown>;
}

function string(value: unknown, name: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${name} must be a non-empty string`);
  }
  return value;
}

function nullableString(value: unknown, name: string): string | null {
  return value === null ? null : string(value, name);
}

function nonNegativeNumber(value: unknown, name: string): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new Error(`${name} must be a non-negative number`);
  }
  return value;
}

function positiveNumber(value: unknown, name: string): number {
  const result = nonNegativeNumber(value, name);
  if (result === 0) throw new Error(`${name} must be positive`);
  return result;
}

function percent(value: unknown, name: string): number {
  const result = nonNegativeNumber(value, name);
  if (result > 100) throw new Error(`${name} must be at most 100`);
  return result;
}

function oneOf<const T extends readonly string[]>(
  value: unknown,
  candidates: T,
  name: string,
): T[number] {
  if (typeof value !== "string" || !candidates.includes(value)) {
    throw new Error(`${name} is unsupported`);
  }
  return value;
}
