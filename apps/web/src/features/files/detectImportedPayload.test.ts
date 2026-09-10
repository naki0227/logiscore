import { describe, expect, it, vi } from "vitest";
import { detectImportedPayload } from "./detectImportedPayload";

const bytes = new Uint8Array([1]);
const failure = () => {
  throw new Error("not this protocol");
};

describe("detectImportedPayload", () => {
  it("prefers a v2 project", () => {
    const text = vi.fn(() => "text");
    const result = detectImportedPayload(bytes, {
      projectV2: () => [
        { name: "a.rs", source: "fn main() {}", extension: ".rs" },
      ],
      projectV1: failure,
      text,
      sourceV2: failure,
      sourceV1: () => ({ source: "source", extension: ".rs" }),
    });
    expect(result.type).toBe("project");
    expect(text).not.toHaveBeenCalled();
  });

  it("falls back to a v1 project", () => {
    expect(
      detectImportedPayload(bytes, {
        projectV2: failure,
        projectV1: () => [
          { name: "legacy.rs", source: "legacy", extension: ".rs" },
        ],
        text: failure,
        sourceV2: failure,
        sourceV1: failure,
      }),
    ).toEqual({
      type: "project",
      version: 1,
      files: [{ name: "legacy.rs", source: "legacy", extension: ".rs" }],
    });
  });

  it("falls through an empty project to v2 text", () => {
    expect(
      detectImportedPayload(bytes, {
        projectV2: () => [],
        projectV1: () => [],
        text: () => "hello",
        sourceV2: failure,
        sourceV1: () => ({ source: "source", extension: ".rs" }),
      }),
    ).toEqual({ type: "text", text: "hello" });
  });

  it("falls through text to a v2 source file", () => {
    expect(
      detectImportedPayload(bytes, {
        projectV2: failure,
        projectV1: failure,
        text: failure,
        sourceV2: () => ({
          filename: "main",
          source: "fn main() {}",
          extension: ".rs",
        }),
        sourceV1: failure,
      }),
    ).toEqual({
      type: "source-file",
      version: 2,
      filename: "main",
      source: "fn main() {}",
      extension: ".rs",
    });
  });

  it("falls back to a v1 source file", () => {
    expect(
      detectImportedPayload(bytes, {
        projectV2: failure,
        projectV1: failure,
        text: failure,
        sourceV2: failure,
        sourceV1: () => ({ source: "fn main() {}", extension: ".rs" }),
      }),
    ).toEqual({
      type: "source-file",
      version: 1,
      source: "fn main() {}",
      extension: ".rs",
    });
  });

  it("rejects an unknown or damaged MIDI", () => {
    expect(() =>
      detectImportedPayload(bytes, {
        projectV2: failure,
        projectV1: failure,
        text: failure,
        sourceV2: failure,
        sourceV1: failure,
      }),
    ).toThrow("Unsupported or damaged");
  });
});
