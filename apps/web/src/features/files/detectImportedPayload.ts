import type { ImportedPayload, ProjectFile } from "../../hooks/useEntropy";

interface ImportDecoders {
  projectV2: (bytes: Uint8Array) => ProjectFile[];
  projectV1: (bytes: Uint8Array) => ProjectFile[];
  text: (bytes: Uint8Array) => string;
  sourceV2: (bytes: Uint8Array) => {
    filename: string;
    source: string;
    extension: string;
  };
  sourceV1: (bytes: Uint8Array) => { source: string; extension: string };
}

export function detectImportedPayload(
  bytes: Uint8Array,
  decoders: ImportDecoders,
): ImportedPayload {
  try {
    const files = decoders.projectV2(bytes);
    if (files.length > 0) return { type: "project", version: 2, files };
  } catch {
    // Continue with v1 Project compatibility.
  }
  try {
    const files = decoders.projectV1(bytes);
    if (files.length > 0) return { type: "project", version: 1, files };
  } catch {
    // Continue through the compatibility order.
  }
  try {
    return { type: "text", text: decoders.text(bytes) };
  } catch {
    // Continue with v2 Source File.
  }
  try {
    return { type: "source-file", version: 2, ...decoders.sourceV2(bytes) };
  } catch {
    // Fall back to v1 Source File.
  }
  try {
    return { type: "source-file", version: 1, ...decoders.sourceV1(bytes) };
  } catch {
    throw new Error("Unsupported or damaged Logiscore MIDI");
  }
}
