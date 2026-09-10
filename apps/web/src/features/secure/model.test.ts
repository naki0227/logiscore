import { describe, expect, it } from "vitest";
import { MAX_PASSWORD_BYTES, validateSecurePassword } from "./model";

describe("validateSecurePassword", () => {
  it("requires a non-empty matching password", () => {
    expect(validateSecurePassword("", "").valid).toBe(false);
    expect(validateSecurePassword("secret", "different").valid).toBe(false);
    expect(validateSecurePassword("secret", "secret").valid).toBe(true);
  });

  it("measures the UTF-8 boundary in bytes", () => {
    const valid = "a".repeat(MAX_PASSWORD_BYTES);
    expect(validateSecurePassword(valid, valid).valid).toBe(true);
    const oversizedUnicode = "🔐".repeat(MAX_PASSWORD_BYTES / 4 + 1);
    const result = validateSecurePassword(oversizedUnicode, oversizedUnicode);
    expect(result.valid).toBe(false);
    expect(result.byteLength).toBe(MAX_PASSWORD_BYTES + 4);
  });
});
