import { describe, expect, it } from "vitest";
import { parseSecurePayload } from "./secure-acoustic-codec";

describe("parseSecurePayload", () => {
  it("accepts each authenticated payload shape", () => {
    expect(parseSecurePayload('{"type":"text","text":"秘密"}')).toEqual({
      type: "text",
      text: "秘密",
    });
    expect(
      parseSecurePayload(
        '{"type":"source-file","version":2,"filename":"main","extension":".rs","source":"fn main() {}"}',
      ),
    ).toMatchObject({ type: "source-file", filename: "main" });
    expect(
      parseSecurePayload(
        '{"type":"project","version":2,"files":[{"name":"src/main.rs","extension":".rs","source":"fn main() {}"}]}',
      ),
    ).toMatchObject({ type: "project", version: 2 });
  });

  it.each([
    "null",
    "{}",
    '{"type":"text","text":1}',
    '{"type":"source-file","version":1,"filename":"x","extension":".rs","source":"x"}',
    '{"type":"project","version":2,"files":[{"name":1}]}',
  ])("rejects malformed external JSON: %s", (json) => {
    expect(() => parseSecurePayload(json)).toThrow(
      "Invalid secure payload response",
    );
  });
});
