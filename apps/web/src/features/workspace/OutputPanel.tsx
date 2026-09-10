import { AnimatePresence, motion } from "framer-motion";
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
  onDownload: () => void;
  onSourceChange: (source: string) => void;
}

export function OutputPanel(props: OutputPanelProps) {
  return (
    <motion.div
      key={`right-${props.uiMode}`}
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: 0.3 }}
    >
      {props.uiMode === "encode" ? (
        <EncodedOutput {...props} />
      ) : (
        <DecodedOutput {...props} />
      )}
    </motion.div>
  );
}

function EncodedOutput(props: OutputPanelProps) {
  return (
    <Panel
      title="MIDI MASTER"
      className="output-panel"
      verified={props.verified}
    >
      <div className="midi-output-container">
        <AnimatePresence mode="wait">
          {props.encodedData ? (
            <motion.div
              key="midi-card"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9 }}
              className="midi-card"
            >
              <div className="midi-card-icon">🎼</div>
              <div className="midi-card-info">
                <div className="midi-filename">
                  {props.filename}.{props.encodedFormat.toLowerCase()}
                </div>
                <div className="midi-filesize">
                  {props.encodedData.length.toLocaleString()} BYTES
                </div>
              </div>
              <button className="btn-download-main" onClick={props.onDownload}>
                DOWNLOAD {props.encodedFormat}
              </button>
              <div className="verification-status">
                {(props.isProject ? props.projectVerified : props.verified)
                  ? "✅ LOSSLESS VERIFIED"
                  : "... UNVERIFIED"}
              </div>
              {props.decodedSource && (
                <div className="mini-preview">
                  <div className="preview-header">DECODED VERIFICATION</div>
                  <pre className="preview-content">
                    {props.decodedSource.slice(0, 100)}...
                  </pre>
                </div>
              )}
            </motion.div>
          ) : (
            <motion.div
              key="placeholder"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="midi-placeholder"
            >
              <div className="placeholder-icon">🎵</div>
              <p>ENCODE YOUR CODE TO MUSIC</p>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </Panel>
  );
}

function DecodedOutput(props: OutputPanelProps) {
  return (
    <Panel
      title="DECODED SOURCE"
      className="editor-panel"
      verified={props.verified}
      headerAction={
        <button
          className="btn-icon"
          onClick={props.onDownload}
          title={`Download ${props.encodedFormat}`}
        >
          <Icons.Download />
        </button>
      }
    >
      <textarea
        className="code-input"
        aria-label="Decoded source"
        value={props.source}
        onChange={(event) => props.onSourceChange(event.target.value)}
        spellCheck={false}
      />
    </Panel>
  );
}
