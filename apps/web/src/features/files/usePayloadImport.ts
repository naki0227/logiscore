import type { SetStatus } from "../status/model";
import type { ChangeEvent, DragEvent } from "react";
import type { ImportedPayload, ProjectFile } from "../../hooks/useEntropy";
import type { PayloadSelection } from "../codec/model";
import type { ModeSelection } from "../codec/model";
import {
  extensionForFile,
  scanProjectEntry,
  SUPPORTED_EXTENSIONS,
} from "./projectFiles";

interface PayloadImportOptions {
  ready: boolean;
  payload: PayloadSelection;
  mode: ModeSelection;
  decodeMidi: (bytes: Uint8Array) => ImportedPayload;
  decodeAudio: (file: File) => Promise<ImportedPayload>;
  stop: () => void;
  setPayload: (payload: PayloadSelection) => void;
  setMode: (mode: ModeSelection) => void;
  setProjectFiles: (files: ProjectFile[]) => void;
  setSource: (source: string) => void;
  setExtension: (extension: string) => void;
  setFilename: (filename: string) => void;
  setUiMode: (mode: "encode" | "decode") => void;
  setStatus: SetStatus;
}

export function usePayloadImport(options: PayloadImportOptions) {
  const applyMidi = async (file: File) => {
    options.stop();
    options.setStatus("MIDIを読み込み中…");
    try {
      const decoded = options.decodeMidi(
        new Uint8Array(await file.arrayBuffer()),
      );
      applyDecodedPayload(decoded, options);
    } catch {
      options.setPayload("source-file");
      options.setProjectFiles([]);
      options.setStatus(
        "LogiscoreのMIDIではないか、ファイルが壊れています",
        "error",
      );
    }
  };

  const applySource = async (file: File, status: string) => {
    const extension = extensionForFile(file.name);
    options.setSource(await file.text());
    options.setPayload("source-file");
    options.setUiMode("encode");
    options.setFilename(
      file.name.replace(/\.[^.]+$/, "") || "logiscore_output",
    );
    if (SUPPORTED_EXTENSIONS.includes(extension))
      options.setExtension(extension);
    options.setStatus(status, "success");
  };

  const applyAudio = async (file: File) => {
    options.stop();
    options.setStatus("録音を解析中…");
    try {
      const decoded = await options.decodeAudio(file);
      options.setMode(options.mode === "secure" ? "secure" : "reliable");
      applyDecodedPayload(decoded, options);
    } catch {
      options.setPayload("source-file");
      options.setProjectFiles([]);
      options.setStatus("Logiscoreの録音として認識できませんでした", "error");
    }
  };

  const handleImport = (event: ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    const file = files?.[0];
    if (!file || !options.ready) return;
    if (files.length === 1 && isMidi(file)) {
      void applyMidi(file);
      return;
    }
    if (files.length === 1 && isAudio(file)) {
      void applyAudio(file);
      return;
    }
    if (options.payload === "project" && files?.length) {
      void applyProjectFiles(files, options);
      return;
    }
    void applySource(file, "ファイルを読み込みました");
  };

  const handleDragOver = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
  };

  const handleDrop = async (event: DragEvent) => {
    handleDragOver(event);
    const entry = event.dataTransfer.items?.[0]?.webkitGetAsEntry();
    if (entry?.isDirectory) {
      options.setStatus("プロジェクトを読み込み中…");
      const files = await scanProjectEntry(entry);
      if (files.length > 0) {
        options.setProjectFiles(files);
        options.setPayload("project");
        options.setFilename(entry.name);
        options.setStatus(
          `プロジェクトを読み込みました: ${files.length} ファイル`,
        );
      }
      return;
    }

    const file = event.dataTransfer.files?.[0];
    if (!file) return;
    if (isMidi(file)) await applyMidi(file);
    else if (isAudio(file)) await applyAudio(file);
    else await applySource(file, "ファイルを読み込みました");
  };

  return { handleImport, handleDragOver, handleDrop };
}

function applyDecodedPayload(
  decoded: ImportedPayload,
  options: PayloadImportOptions,
) {
  options.setPayload(decoded.type);
  if (decoded.type === "project") {
    options.setProjectFiles(decoded.files);
    options.setUiMode("encode");
    options.setStatus(
      `プロジェクトを復元しました: ${decoded.files.length} ファイル`,
      "success",
    );
    return;
  }

  options.setSource(decoded.type === "text" ? decoded.text : decoded.source);
  if (decoded.type === "source-file") {
    options.setExtension(decoded.extension);
    if (decoded.filename) options.setFilename(decoded.filename);
  }
  options.setUiMode("decode");
  options.setStatus(
    decoded.type === "text"
      ? "テキストを復元しました"
      : "ソースファイルを復元しました",
    "success",
  );
}

async function applyProjectFiles(
  selectedFiles: FileList,
  options: PayloadImportOptions,
) {
  options.setStatus("プロジェクトを読み込み中…");
  const files = (
    await Promise.all(
      Array.from(selectedFiles).map(
        async (file): Promise<ProjectFile | null> => {
          const extension = extensionForFile(file.name);
          if (!SUPPORTED_EXTENSIONS.includes(extension)) return null;
          return {
            name: file.webkitRelativePath || file.name,
            extension,
            source: await file.text(),
          };
        },
      ),
    )
  ).filter((file): file is ProjectFile => file !== null);
  if (files.length === 0) {
    options.setStatus("対応しているファイルが見つかりません", "error");
    return;
  }
  const relativeRoot = files[0]?.name.split("/")[0];
  options.setProjectFiles(files);
  options.setFilename(relativeRoot || "logiscore_project");
  options.setUiMode("encode");
  options.setStatus(`プロジェクトを読み込みました: ${files.length} ファイル`);
}

function isMidi(file: File): boolean {
  const name = file.name.toLowerCase();
  return name.endsWith(".mid") || name.endsWith(".midi");
}

function isAudio(file: File): boolean {
  const name = file.name.toLowerCase();
  return (
    file.type.startsWith("audio/") ||
    [".wav", ".m4a", ".mp3", ".opus", ".ogg", ".aac"].some((extension) =>
      name.endsWith(extension),
    )
  );
}
