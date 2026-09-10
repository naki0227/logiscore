import { describe, expect, it } from "vitest";
import { mixChannels } from "./decodeAudioFile";

describe("mixChannels", () => {
  it("downmixes stereo PCM without changing frame count", () => {
    expect(
      Array.from(
        mixChannels([
          new Float32Array([1, -1, 0.5]),
          new Float32Array([-1, 1, 0.5]),
        ]),
      ),
    ).toEqual([0, 0, 0.5]);
  });

  it("rejects empty and mismatched channels", () => {
    expect(() => mixChannels([])).toThrow("matching");
    expect(() =>
      mixChannels([new Float32Array(1), new Float32Array(2)]),
    ).toThrow("matching");
  });
});
