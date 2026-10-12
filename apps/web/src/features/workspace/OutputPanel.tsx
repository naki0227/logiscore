import type { ReactNode } from "react";
import { Icons } from "../../components/Icons";
import Panel from "../../components/Panel";

interface OutputPanelProps {
  uiMode: "encode" | "decode";
  filename: string;
  encodedData: Uint8Array | null;
  encodedFormat: "MIDI" | "WAV";
  decodedSource: string | null;
  source: string;
  verified: boolean;
  projectVerified: boolean;
  isProject: boolean;
  score: ReactNode;
  onDownload: () => void;
  onSourceChange: (source: string) => void;
}

export function OutputPanel(props: OutputPanelProps) {
  return (
    <div>
      {props.uiMode === "encode" ? (
        <EncodedOutput {...props} />
      ) : (
        <DecodedOutput {...props} />
      )}
    </div>
  );
}

function EncodedOutput(props: OutputPanelProps) {
  const verified = props.isProject ? props.projectVerified : props.verified;
  return (
    <Panel title={`出力 (${props.encodedFormat})`} className="output-panel">
      {props.score}
      <div className="output-body">
        {props.encodedData ? (
          <>
            <dl className="output-meta">
              <dt>ファイル名</dt>
              <dd className="midi-filename">
                {props.filename}.{props.encodedFormat.toLowerCase()}
              </dd>
              <dt>サイズ</dt>
              <dd>{props.encodedData.length.toLocaleString()} バイト</dd>
              <dt>復元の確認</dt>
              <dd
                className={
                  verified ? "verification-status ok" : "verification-status"
                }
              >
                {verified ? "元のデータに戻せることを確認しました" : "未確認"}
              </dd>
            </dl>
            <button
              className="btn btn-download-main"
              onClick={props.onDownload}
            >
              {props.encodedFormat}をダウンロード
            </button>
            {props.decodedSource && (
              <div className="mini-preview">
                <p className="preview-header">復元結果の先頭</p>
                <pre className="preview-content">
                  {props.decodedSource.slice(0, 100)}
                  {props.decodedSource.length > 100 && "…"}
                </pre>
              </div>
            )}
          </>
        ) : (
          <p className="midi-placeholder">
            「変換する」を押すと、ここに{props.encodedFormat}
            ファイルが表示されます。
          </p>
        )}
      </div>
    </Panel>
  );
}

function DecodedOutput(props: OutputPanelProps) {
  return (
    <Panel
      title="復元結果"
      className="editor-panel"
      verified={props.verified}
      headerAction={
        <button
          className="btn-icon"
          onClick={props.onDownload}
          title={`${props.encodedFormat}をダウンロード`}
        >
          <Icons.Download />
        </button>
      }
    >
      {props.score}
      <textarea
        className="code-input"
        aria-label="復元したデータ"
        value={props.source}
        onChange={(event) => props.onSourceChange(event.target.value)}
        spellCheck={false}
      />
    </Panel>
  );
}
