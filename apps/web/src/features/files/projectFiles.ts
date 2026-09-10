import type { ProjectFile } from "../../hooks/useEntropy";

export const SUPPORTED_EXTENSIONS = [
  ".rs",
  ".py",
  ".ts",
  ".js",
  ".tsx",
  ".jsx",
  ".go",
  ".cpp",
  ".c",
  ".h",
  ".hpp",
  ".dart",
  ".swift",
  ".java",
  ".kt",
  ".kts",
  ".rb",
  ".sh",
  ".bash",
  ".zsh",
  ".css",
  ".scss",
  ".sass",
  ".less",
  ".html",
  ".md",
  ".json",
  ".yaml",
  ".yml",
  ".toml",
  ".xml",
  ".svg",
  ".sql",
  ".env",
  ".txt",
  "Dockerfile",
  "Makefile",
  "Gemfile",
  "go.mod",
  "Cargo.toml",
  "package.json",
];

const DEFAULT_IGNORES = [
  ".git",
  "node_modules",
  "dist",
  "build",
  "target",
  "pkg",
  ".DS_Store",
  ".gemini",
  ".cursor",
];

export function extensionForFile(name: string): string {
  return name.includes(".") ? `.${name.split(".").pop()}` : name;
}

export async function scanProjectEntry(
  entry: FileSystemEntry,
  path = "",
  parentIgnores = DEFAULT_IGNORES,
): Promise<ProjectFile[]> {
  if (entry.isFile) {
    const fileEntry = entry as FileSystemFileEntry;
    const file = await new Promise<File>((resolve) => fileEntry.file(resolve));
    const extension = extensionForFile(file.name);
    if (!SUPPORTED_EXTENSIONS.includes(extension)) return [];
    return [{ name: path + file.name, source: await file.text(), extension }];
  }

  if (!entry.isDirectory) return [];
  const directory = entry as FileSystemDirectoryEntry;
  const entries = await new Promise<FileSystemEntry[]>((resolve) =>
    directory.createReader().readEntries(resolve),
  );
  const ignores = await readIgnoreRules(entries, parentIgnores);
  const files: ProjectFile[] = [];

  for (const child of entries) {
    if (child.name !== ".gitignore" && isIgnored(child.name, ignores)) continue;
    files.push(
      ...(await scanProjectEntry(child, path + entry.name + "/", ignores)),
    );
  }
  return files;
}

function isIgnored(name: string, patterns: string[]): boolean {
  return patterns.some((pattern) => {
    const normalized = pattern.endsWith("/") ? pattern.slice(0, -1) : pattern;
    return (
      normalized === "*" ||
      name === normalized ||
      name.startsWith(`${normalized}/`)
    );
  });
}

async function readIgnoreRules(
  entries: FileSystemEntry[],
  parentIgnores: string[],
): Promise<string[]> {
  const gitignore = entries.find((entry) => entry.name === ".gitignore") as
    FileSystemFileEntry | undefined;
  if (!gitignore) return [...parentIgnores];

  try {
    const file = await new Promise<File>((resolve) => gitignore.file(resolve));
    const rules = (await file.text())
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("#"));
    return Array.from(new Set([...parentIgnores, ...rules]));
  } catch (cause) {
    console.warn("Failed to read .gitignore", cause);
    return [...parentIgnores];
  }
}
