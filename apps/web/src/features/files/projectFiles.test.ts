import { describe, expect, it } from "vitest";
import { extensionForFile, SUPPORTED_EXTENSIONS } from "./projectFiles";

describe("extensionForFile", () => {
  it("extracts a dotted extension", () => {
    expect(extensionForFile("main.rs")).toBe(".rs");
  });

  it("preserves supported extensionless filenames", () => {
    expect(extensionForFile("Dockerfile")).toBe("Dockerfile");
  });

  it("keeps the scanner allowlist explicit", () => {
    expect(SUPPORTED_EXTENSIONS).toContain(".txt");
    expect(SUPPORTED_EXTENSIONS).not.toContain(".exe");
  });
});
