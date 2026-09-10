export type AcousticEnvironment =
  "auto" | "quiet" | "conversation" | "noisy" | "online" | "long-distance";

export interface AcousticSettings {
  environment: AcousticEnvironment;
  reliabilityPriority: number;
}

export interface AcousticProfileDescription {
  id: number;
  name: string;
  detected_environment: string;
  confidence_percent: number;
  timing_percent: number;
  max_polyphony: number;
  fec_profile: number;
  interleave_depth: number;
  repetition: number;
  music_weight: number;
  estimated_duration_ms: number;
  fallback: number[];
}

export const DEFAULT_ACOUSTIC_SETTINGS: AcousticSettings = {
  environment: "auto",
  reliabilityPriority: 70,
};

export function parseProfileDescription(
  json: string,
): AcousticProfileDescription {
  const value: unknown = JSON.parse(json);
  if (!isRecord(value) || !isProfile(value)) {
    throw new Error("Invalid acoustic profile response");
  }
  return value;
}

export function formatDuration(milliseconds: number): string {
  if (!Number.isFinite(milliseconds) || milliseconds < 0) return "—";
  const seconds = Math.ceil(milliseconds / 1_000);
  if (seconds < 60) return `${seconds} sec`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes} min ${seconds % 60} sec`;
}

function isProfile(
  value: Record<string, unknown>,
): value is Record<string, unknown> & AcousticProfileDescription {
  return (
    typeof value.id === "number" &&
    typeof value.name === "string" &&
    typeof value.detected_environment === "string" &&
    typeof value.confidence_percent === "number" &&
    typeof value.timing_percent === "number" &&
    typeof value.max_polyphony === "number" &&
    typeof value.fec_profile === "number" &&
    typeof value.interleave_depth === "number" &&
    typeof value.repetition === "number" &&
    typeof value.music_weight === "number" &&
    typeof value.estimated_duration_ms === "number" &&
    Array.isArray(value.fallback) &&
    value.fallback.every((entry) => typeof entry === "number")
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
