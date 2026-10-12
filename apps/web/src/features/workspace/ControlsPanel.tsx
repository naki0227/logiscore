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
    <div>
      <Panel title="操作" className="controls-panel">
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
            <Info label="言語" value={props.extensionInfo.name} />
            <Info label="音階" value={props.extensionInfo.scale_name} />
            <Info
              label="主音"
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
            {props.processing ? <div className="spinner-small" /> : "変換する"}
          </button>
          <button
            className="btn btn-play"
            onClick={props.onPlay}
            disabled={props.encodedSize === null || props.playing}
          >
            <Icons.Play /> 再生
          </button>
          <button
            className="btn btn-stop"
            onClick={props.onStop}
            disabled={!props.playing}
          >
            <Icons.Stop /> 停止
          </button>
        </div>
        <div className="progress-container">
          <div className="progress-label">再生の進み具合</div>
          <div className="progress-bar-container">
            <div
              className="progress-bar"
              style={{ width: `${props.progress}%` }}
            />
          </div>
        </div>
        {props.encodedSize !== null && (
          <div className="midi-info">
            {props.encodedFormat}: {props.encodedSize.toLocaleString()} バイト
          </div>
        )}
      </Panel>
    </div>
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
