import { motion } from "framer-motion";
import { Icons } from "../../components/Icons";
import Panel from "../../components/Panel";
import { CodecSelector } from "../codec/CodecSelector";
import { AcousticProfileControls } from "../acoustic/AcousticProfileControls";
import { SecurePasswordControls } from "../secure/SecurePasswordControls";
import type { SecurePasswordState } from "../secure/model";
import type {
  AcousticEnvironment,
  AcousticProfileDescription,
} from "../acoustic/model";
import {
  protocolLabel,
  type ModeSelection,
  type PayloadSelection,
} from "../codec/model";

interface ExtensionInfo {
  name: string;
  scale_name: string;
  root_key: number;
}
interface ControlsPanelProps {
  payload: PayloadSelection;
  mode: ModeSelection;
  v2Version: number | null;
  extensionInfo: ExtensionInfo | null;
  processing: boolean;
  ready: boolean;
  playing: boolean;
  progress: number;
  encodedSize: number | null;
  encodedFormat: "MIDI" | "WAV";
  acousticEnvironment: AcousticEnvironment;
  reliabilityPriority: number;
  acousticProfile: AcousticProfileDescription | null;
  securePassword: string;
  secureConfirmation: string;
  secureValidation: SecurePasswordState;
  onPayloadChange: (payload: PayloadSelection) => void;
  onModeChange: (mode: ModeSelection) => void;
  onEnvironmentChange: (environment: AcousticEnvironment) => void;
  onPriorityChange: (priority: number) => void;
  onSecurePasswordChange: (password: string) => void;
  onSecureConfirmationChange: (confirmation: string) => void;
  onEncode: () => void;
  onPlay: () => void;
  onStop: () => void;
}

const NOTES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

export function ControlsPanel(props: ControlsPanelProps) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay: 0.2 }}
    >
      <Panel title="CONTROLS" className="controls-panel">
        <CodecSelector
          payload={props.payload}
          mode={props.mode}
          protocol={protocolLabel(props.payload, props.mode, props.v2Version)}
          onPayloadChange={props.onPayloadChange}
          onModeChange={props.onModeChange}
        />
        {(props.mode === "reliable" || props.mode === "secure") && (
          <AcousticProfileControls
            environment={props.acousticEnvironment}
            reliabilityPriority={props.reliabilityPriority}
            profile={props.acousticProfile}
            onEnvironmentChange={props.onEnvironmentChange}
            onPriorityChange={props.onPriorityChange}
          />
        )}
        {props.mode === "secure" && (
          <SecurePasswordControls
            password={props.securePassword}
            confirmation={props.secureConfirmation}
            validation={props.secureValidation}
            onPasswordChange={props.onSecurePasswordChange}
            onConfirmationChange={props.onSecureConfirmationChange}
          />
        )}
        {props.extensionInfo && (
          <div className="info-display">
            <Info label="Language" value={props.extensionInfo.name} />
            <Info label="Scale" value={props.extensionInfo.scale_name} />
            <Info
              label="Root"
              value={NOTES[props.extensionInfo.root_key] ?? "-"}
            />
          </div>
        )}
        <div className="button-group">
          <button
            className={`btn btn-encode ${props.processing ? "shimmer" : ""}`}
            onClick={props.onEncode}
            disabled={
              !props.ready ||
              props.playing ||
              props.processing ||
              (props.mode === "secure" && !props.secureValidation.valid)
            }
          >
            {props.processing ? <div className="spinner-small" /> : "ENCODE"}
          </button>
          <button
            className="btn btn-play"
            onClick={props.onPlay}
            disabled={props.encodedSize === null || props.playing}
          >
            <Icons.Play /> PLAY
          </button>
          <button
            className="btn btn-stop"
            onClick={props.onStop}
            disabled={!props.playing}
          >
            <Icons.Stop /> STOP
          </button>
        </div>
        <div className="progress-container">
          <div className="progress-label">PROGRESS</div>
          <div className="progress-bar-container">
            <motion.div
              className="progress-bar"
              animate={{ width: `${props.progress}%` }}
              transition={{ type: "spring", bounce: 0, duration: 0.3 }}
            />
          </div>
        </div>
        {props.encodedSize !== null && (
          <div className="midi-info">
            {props.encodedFormat}: {props.encodedSize.toLocaleString()} BYTES
          </div>
        )}
      </Panel>
    </motion.div>
  );
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-row">
      <span className="info-label">{label}</span>
      <span className="info-value">{value}</span>
    </div>
  );
}
