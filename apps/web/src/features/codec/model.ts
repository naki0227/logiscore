export type PayloadSelection = "text" | "source-file" | "project";
export type ModeSelection = "auto" | "music" | "reliable" | "secure" | "fast";
export type CodecRoute =
  | "v2-text-dense"
  | "v2-file-dense"
  | "v2-project-dense"
  | "v2-text-musical"
  | "v2-file-musical"
  | "v2-project-musical"
  | "v2-text-reliable"
  | "v2-file-reliable"
  | "v2-project-reliable"
  | "v2-text-secure"
  | "v2-file-secure"
  | "v2-project-secure";

export function resolveCodecRoute(
  payload: PayloadSelection,
  mode: ModeSelection,
): CodecRoute {
  if (mode === "secure") {
    switch (payload) {
      case "text":
        return "v2-text-secure";
      case "source-file":
        return "v2-file-secure";
      case "project":
        return "v2-project-secure";
    }
  }

  if (mode === "reliable") {
    switch (payload) {
      case "text":
        return "v2-text-reliable";
      case "source-file":
        return "v2-file-reliable";
      case "project":
        return "v2-project-reliable";
    }
  }

  if (mode === "music") {
    switch (payload) {
      case "text":
        return "v2-text-musical";
      case "source-file":
        return "v2-file-musical";
      case "project":
        return "v2-project-musical";
    }
  }

  switch (payload) {
    case "text":
      return "v2-text-dense";
    case "source-file":
      return "v2-file-dense";
    case "project":
      return "v2-project-dense";
  }
}

export function protocolLabel(
  _payload: PayloadSelection,
  mode: ModeSelection,
  v2Version: number | null,
): string {
  const profile =
    mode === "music"
      ? "RHYTHMIC"
      : mode === "secure"
        ? "PCM + AEAD + FEC"
        : mode === "reliable"
          ? "PCM + FEC"
          : "DENSE";
  return `V${v2Version ?? "-"} / ${profile}`;
}

interface ComparableProjectFile {
  name: string;
  extension: string;
  source: string;
}

export function areProjectFilesEqual(
  left: readonly ComparableProjectFile[],
  right: readonly ComparableProjectFile[],
): boolean {
  if (left.length !== right.length) return false;
  const byPath = (a: ComparableProjectFile, b: ComparableProjectFile) =>
    a.name < b.name ? -1 : a.name > b.name ? 1 : 0;
  const sortedLeft = [...left].sort(byPath);
  const sortedRight = [...right].sort(byPath);
  return sortedLeft.every((file, index) => {
    const other = sortedRight[index];
    return (
      file.name === other?.name &&
      file.extension === other.extension &&
      file.source === other.source
    );
  });
}
