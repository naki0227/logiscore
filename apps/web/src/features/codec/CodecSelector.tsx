import {
  modeDescription,
  type ModeSelection,
  type PayloadSelection,
} from "./model";

interface CodecSelectorProps {
  payload: PayloadSelection;
  mode: ModeSelection;
  protocol: string;
  onPayloadChange: (payload: PayloadSelection) => void;
  onModeChange: (mode: ModeSelection) => void;
}

export function CodecSelector({
  payload,
  mode,
  protocol,
  onPayloadChange,
  onModeChange,
}: CodecSelectorProps) {
  return (
    <div className="codec-selector" aria-label="v2 codec settings">
      <label>
        <span>Payload</span>
        <select
          value={payload}
          onChange={(event) =>
            onPayloadChange(event.target.value as PayloadSelection)
          }
        >
          <option value="text">Text</option>
          <option value="source-file">Source File</option>
          <option value="project">Project</option>
        </select>
      </label>
      <label>
        <span>Mode</span>
        <select
          value={mode}
          onChange={(event) =>
            onModeChange(event.target.value as ModeSelection)
          }
        >
          <option value="auto">Auto · MIDI</option>
          <option value="fast">Fast · MIDI</option>
          <option value="music">Music · MIDI</option>
          <option value="reliable">Reliable · WAV</option>
          <option value="secure">Secure · WAV</option>
        </select>
      </label>
      <p className="mode-hint">{modeDescription(mode)}</p>
      <div className="codec-protocol">{protocol}</div>
    </div>
  );
}
