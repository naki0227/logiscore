import { describe, expect, it } from "vitest";
import { parseCoreRecordingTelemetry } from "./recordingMetricsModel";

describe("parseCoreRecordingTelemetry", () => {
  it("maps a complete Rust telemetry response", () => {
    expect(
      parseCoreRecordingTelemetry(
        JSON.stringify({
          profile_id: 7,
          final_recovery_rate_percent: 100,
          raw_symbol_accuracy_percent: 99.5,
          corrected_errors: 3,
        }),
      ),
    ).toEqual({
      profileId: 7,
      finalRecoveryRatePercent: 100,
      rawSymbolAccuracyPercent: 99.5,
      correctedErrors: 3,
    });
  });

  it("rejects out-of-range and malformed telemetry", () => {
    expect(() =>
      parseCoreRecordingTelemetry(
        JSON.stringify({
          profile_id: 8,
          final_recovery_rate_percent: 100,
          raw_symbol_accuracy_percent: 100,
          corrected_errors: 0,
        }),
      ),
    ).toThrow("profile");
    expect(() => parseCoreRecordingTelemetry("{}")).toThrow("profile");
  });
});
