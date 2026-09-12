export interface CoreRecordingTelemetry {
  profileId: number | null;
  finalRecoveryRatePercent: number;
  rawSymbolAccuracyPercent: number;
  correctedErrors: number;
}

export function parseCoreRecordingTelemetry(
  input: string,
): CoreRecordingTelemetry {
  const value: unknown = JSON.parse(input);
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Invalid recording telemetry");
  }
  const record = value as Record<string, unknown>;
  const profileId = record.profile_id;
  if (
    profileId !== null &&
    (typeof profileId !== "number" ||
      !Number.isInteger(profileId) ||
      profileId < 1 ||
      profileId > 7)
  ) {
    throw new Error("Invalid recording telemetry profile");
  }
  return {
    profileId,
    finalRecoveryRatePercent: percentage(
      record.final_recovery_rate_percent,
      "final recovery rate",
    ),
    rawSymbolAccuracyPercent: percentage(
      record.raw_symbol_accuracy_percent,
      "raw symbol accuracy",
    ),
    correctedErrors: nonNegativeInteger(
      record.corrected_errors,
      "corrected errors",
    ),
  };
}

function percentage(value: unknown, name: string): number {
  if (
    typeof value !== "number" ||
    !Number.isFinite(value) ||
    value < 0 ||
    value > 100
  ) {
    throw new Error(`Invalid recording telemetry ${name}`);
  }
  return value;
}

function nonNegativeInteger(value: unknown, name: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new Error(`Invalid recording telemetry ${name}`);
  }
  return value;
}
