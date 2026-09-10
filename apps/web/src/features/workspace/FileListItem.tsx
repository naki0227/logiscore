import { memo, type ReactNode } from "react";
import { Icons } from "../../components/Icons";
import type { ProjectFile } from "../../hooks/useEntropy";

export const FileListItem = memo(function FileListItem({
  file,
  isActive,
  rootName,
}: {
  file: ProjectFile;
  isActive: boolean;
  rootName: string;
}) {
  return (
    <div className={`file-item ${isActive ? "playing" : ""}`}>
      <span className="file-icon">
        {isActive ? <Icons.Play /> : <Icons.Check />}
      </span>
      <span className="file-name">{formatPath(file.name, rootName)}</span>
      <span className="file-ext">{file.extension}</span>
    </div>
  );
});

function formatPath(fullPath: string, rootName: string): ReactNode {
  const relative = fullPath.startsWith(`${rootName}/`)
    ? fullPath.slice(rootName.length + 1)
    : fullPath;
  const parts = relative.split("/");
  const fileName = parts.pop();
  if (parts.length === 0)
    return <span className="file-basename">{fileName}</span>;
  const displayDirectory =
    parts.length > 3 ? `.../${parts.slice(-2).join("/")}` : parts.join("/");
  return (
    <span className="file-name-container">
      <span className="file-path">{displayDirectory}/</span>
      <span className="file-basename">{fileName}</span>
    </span>
  );
}
