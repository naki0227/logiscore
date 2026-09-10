import type { ChangeEvent } from "react";
import { motion } from "framer-motion";
import { Icons } from "../../components/Icons";
import Panel from "../../components/Panel";
import type { ProjectFile } from "../../hooks/useEntropy";
import type { PayloadSelection } from "../codec/model";
import { SUPPORTED_EXTENSIONS } from "../files/projectFiles";
import { FileListItem } from "./FileListItem";

interface SourcePanelProps {
  uiMode: "encode" | "decode";
  payload: PayloadSelection;
  source: string;
  extension: string;
  filename: string;
  systemVersion: string | null;
  projectFiles: ProjectFile[];
  activeFile: string | null;
  inputSize: number;
  inputFormat: "MIDI" | "WAV";
  onPayloadChange: (payload: PayloadSelection) => void;
  onSourceChange: (source: string) => void;
  onExtensionChange: (extension: string) => void;
  onUiModeChange: (mode: "encode" | "decode") => void;
  onImport: (event: ChangeEvent<HTMLInputElement>) => void;
  onDownloadZip: () => void;
}

export function SourcePanel(props: SourcePanelProps) {
  const isProject = props.payload === "project";
  const isText = props.payload === "text";
  return (
    <motion.div
      key={`left-${props.uiMode}`}
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: 0.1 }}
    >
      {props.uiMode === "encode" ? (
        <Panel
          title={
            isProject
              ? "PROJECT RADIUS"
              : isText
                ? "TEXT PAYLOAD"
                : "SOURCE CODE"
          }
          className="editor-panel"
          headerAction={
            <SourceActions {...props} isProject={isProject} isText={isText} />
          }
        >
          {isProject ? (
            <ProjectFiles {...props} />
          ) : (
            <SourceEditor {...props} isText={isText} />
          )}
        </Panel>
      ) : (
        <Panel
          title={`INPUT ${props.inputFormat}`}
          className="input-panel-midi"
          headerAction={<ImportButton onImport={props.onImport} />}
        >
          <div className="midi-output-container">
            <div className="midi-card input-version">
              <div className="midi-card-icon">
                <Icons.Node />
              </div>
              <div className="midi-card-info">
                <div className="midi-filename">
                  Imported {props.inputFormat} Data
                </div>
                <div className="midi-filesize">
                  {props.inputSize.toLocaleString()} BYTES
                </div>
              </div>
              <div className="input-label">READY TO DECODE</div>
              <button
                className="btn-icon circle-large"
                onClick={() => props.onUiModeChange("encode")}
              >
                <Icons.Edit />
              </button>
              <p className="hint-text">Click to edit source code</p>
            </div>
          </div>
        </Panel>
      )}
    </motion.div>
  );
}

function SourceActions(
  props: SourcePanelProps & { isProject: boolean; isText: boolean },
) {
  return (
    <div className="header-actions">
      <button
        className={`btn-icon ${props.isProject ? "active" : ""}`}
        title="Project Mode"
        onClick={() =>
          props.onPayloadChange(props.isProject ? "source-file" : "project")
        }
      >
        <Icons.Node />
      </button>
      {props.isProject ? (
        <>
          <ProjectFolderButton onImport={props.onImport} />
          <ImportButton onImport={props.onImport} midiOnly />
        </>
      ) : (
        <ImportButton onImport={props.onImport} />
      )}
      {!props.isProject && !props.isText && (
        <select
          className="ext-select"
          value={props.extension}
          onChange={(event) => props.onExtensionChange(event.target.value)}
        >
          {SUPPORTED_EXTENSIONS.map((extension) => (
            <option key={extension} value={extension}>
              {extension}
            </option>
          ))}
        </select>
      )}
    </div>
  );
}

function ImportButton({
  onImport,
  midiOnly = false,
}: {
  onImport: (event: ChangeEvent<HTMLInputElement>) => void;
  midiOnly?: boolean;
}) {
  return (
    <label
      className="btn-icon"
      title={midiOnly ? "Import MIDI or recording" : "Import File"}
    >
      <Icons.Import />
      <input
        type="file"
        aria-label={midiOnly ? "Import MIDI or recording" : "Import file"}
        accept={
          midiOnly
            ? ".mid,.midi,.wav,.m4a,.mp3,.opus,.ogg,.aac"
            : ".mid,.midi,.wav,.m4a,.mp3,.opus,.ogg,.aac,.rs,.py,.ts,.go,.cpp,.rb,.css,.md,.json,.yaml,.toml,.txt"
        }
        hidden
        onChange={onImport}
      />
    </label>
  );
}

function ProjectFolderButton({
  onImport,
}: {
  onImport: (event: ChangeEvent<HTMLInputElement>) => void;
}) {
  return (
    <label className="btn-icon" title="Import Project Folder">
      <Icons.Node />
      <input
        type="file"
        aria-label="Import Project folder"
        multiple
        {...{ webkitdirectory: "" }}
        hidden
        onChange={onImport}
      />
    </label>
  );
}

function ProjectFiles(props: SourcePanelProps) {
  return (
    <div className="project-file-list">
      <div
        className="project-header"
        style={{ display: "flex", alignItems: "center", gap: "8px" }}
      >
        <span className="project-name">{props.filename}</span>
        <span className="project-count">
          {props.projectFiles.length} tracks
        </span>
        {props.systemVersion && (
          <span
            className="project-version"
            style={{ fontSize: "10px", opacity: 0.5 }}
          >
            Protocol v{props.systemVersion}
          </span>
        )}
        {props.projectFiles.length > 0 && (
          <button
            className="btn-icon"
            onClick={props.onDownloadZip}
            title="Download Source ZIP"
            style={{ marginLeft: "auto" }}
          >
            <Icons.Download /> ZIP
          </button>
        )}
      </div>
      <div className="file-items">
        {props.projectFiles.map((file) => (
          <FileListItem
            key={file.name}
            file={file}
            isActive={props.activeFile === file.name}
            rootName={props.filename}
          />
        ))}
        {props.projectFiles.length === 0 && (
          <div className="project-placeholder">
            Drop a folder here to start Symphony
          </div>
        )}
      </div>
    </div>
  );
}

function SourceEditor(props: SourcePanelProps & { isText: boolean }) {
  return (
    <textarea
      className="code-input"
      aria-label={props.isText ? "Text payload" : "Source code"}
      value={props.source}
      onChange={(event) => {
        props.onSourceChange(event.target.value);
        props.onUiModeChange("encode");
      }}
      spellCheck={false}
      placeholder={
        props.isText
          ? "Type a message to transmit..."
          : "Paste your source code here..."
      }
    />
  );
}
