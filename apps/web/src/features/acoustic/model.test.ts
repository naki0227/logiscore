import { describe, expect, it } from "vitest";
import { formatDuration, parseProfileDescription } from "./model";

describe("adaptive acoustic model", () => {
  it("parses a complete profile boundary response", () => {
    const profile = parseProfileDescription(
      JSON.stringify({
        id: 2,
        name: "Balanced",
        detected_environment: "Auto",
        confidence_percent: 50,
        timing_percent: 100,
        max_polyphony: 4,
        fec_profile: 1,
        interleave_depth: 8,
        repetition: 3,
        music_weight: 60,
        estimated_duration_ms: 42_000,
        fallback: [4, 6, 7],
      }),
    );
    expect(profile.name).toBe("Balanced");
    expect(profile.fallback).toEqual([4, 6, 7]);
  });

  it("rejects malformed external data", () => {
    expect(() => parseProfileDescription('{"name":"Balanced"}')).toThrow(
      "Invalid acoustic profile response",
    );
  });

  it("formats seconds and minute boundaries", () => {
    expect(formatDuration(42_000)).toBe("42 sec");
    expect(formatDuration(61_001)).toBe("1 min 2 sec");
    expect(formatDuration(Number.NaN)).toBe("—");
  });
});
