import { describe, expect, it } from "vitest";
import {
  areProjectFilesEqual,
  protocolLabel,
  resolveCodecRoute,
} from "./model";

describe("resolveCodecRoute", () => {
  it.each([
    ["text", "v2-text-dense"],
    ["source-file", "v2-file-dense"],
    ["project", "v2-project-dense"],
  ] as const)("routes %s through %s in auto mode", (payload, expected) => {
    expect(resolveCodecRoute(payload, "auto")).toBe(expected);
  });

  it("maps fast text mode to the deterministic v2 dense codec", () => {
    expect(resolveCodecRoute("text", "fast")).toBe("v2-text-dense");
  });

  it.each([
    ["text", "v2-text-musical"],
    ["source-file", "v2-file-musical"],
    ["project", "v2-project-musical"],
  ] as const)("routes %s through %s in music mode", (payload, expected) => {
    expect(resolveCodecRoute(payload, "music")).toBe(expected);
  });

  it.each([
    ["text", "v2-text-reliable"],
    ["source-file", "v2-file-reliable"],
    ["project", "v2-project-reliable"],
  ] as const)("routes %s through %s in reliable mode", (payload, expected) => {
    expect(resolveCodecRoute(payload, "reliable")).toBe(expected);
  });

  it.each([
    ["text", "v2-text-secure"],
    ["source-file", "v2-file-secure"],
    ["project", "v2-project-secure"],
  ] as const)("routes %s through %s in secure mode", (payload, expected) => {
    expect(resolveCodecRoute(payload, "secure")).toBe(expected);
  });
});

describe("protocolLabel", () => {
  it("shows the binary packet version for text", () => {
    expect(protocolLabel("text", "auto", 2)).toBe("V2 / DENSE");
  });

  it("shows v2 for source files and projects", () => {
    expect(protocolLabel("source-file", "fast", 2)).toBe("V2 / DENSE");
    expect(protocolLabel("project", "auto", 2)).toBe("V2 / DENSE");
  });

  it("shows rhythmic profile when Music mode is selected", () => {
    expect(protocolLabel("text", "music", 2)).toBe("V2 / RHYTHMIC");
  });

  it("shows PCM and FEC when Reliable mode is selected", () => {
    expect(protocolLabel("text", "reliable", 2)).toBe("V2 / PCM + FEC");
  });

  it("shows authenticated encryption when Secure mode is selected", () => {
    expect(protocolLabel("text", "secure", 2)).toBe("V2 / PCM + AEAD + FEC");
  });
});

describe("areProjectFilesEqual", () => {
  const main = {
    name: "src/main.rs",
    extension: ".rs",
    source: "fn main() {}",
  };
  const readme = { name: "README.md", extension: ".md", source: "# Project" };

  it("compares canonical content independently of selection order", () => {
    expect(areProjectFilesEqual([main, readme], [readme, main])).toBe(true);
  });

  it("detects path, extension, source, and file-count differences", () => {
    expect(areProjectFilesEqual([main], [])).toBe(false);
    expect(areProjectFilesEqual([main], [{ ...main, name: "lib.rs" }])).toBe(
      false,
    );
    expect(areProjectFilesEqual([main], [{ ...main, extension: ".txt" }])).toBe(
      false,
    );
    expect(areProjectFilesEqual([main], [{ ...main, source: "changed" }])).toBe(
      false,
    );
  });
});
