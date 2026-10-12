import { describe, expect, it } from "vitest";
import { previewPitches, SCORE_NOTE_LIMIT, staffStep } from "./model";

describe("score model", () => {
  it("places E4 on the bottom line and F5 on the top line", () => {
    expect(staffStep(64)).toBe(0);
    expect(staffStep(65)).toBe(1);
    expect(staffStep(77)).toBe(8);
  });

  it("folds pitches outside the staff back into its nine steps", () => {
    for (const pitch of [0, 21, 40, 60, 90, 108, 127]) {
      const step = staffStep(pitch);
      expect(step).toBeGreaterThanOrEqual(0);
      expect(step).toBeLessThanOrEqual(8);
    }
  });

  it("turns each input byte into one note and caps the count", () => {
    expect(previewPitches("ab")).toHaveLength(2);
    expect(previewPitches("あ")).toHaveLength(3); // UTF-8 3 bytes
    expect(previewPitches("x".repeat(100))).toHaveLength(SCORE_NOTE_LIMIT);
    expect(previewPitches("")).toEqual([]);
  });

  it("is deterministic so typing the same text gives the same score", () => {
    expect(previewPitches("Logiscore")).toEqual(previewPitches("Logiscore"));
  });
});
