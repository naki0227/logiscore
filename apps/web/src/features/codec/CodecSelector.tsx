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
    <div className="codec-selector" aria-label="変換の設定">
      <label>
        <span>送るもの</span>
        <select
          value={payload}
          onChange={(event) =>
            onPayloadChange(event.target.value as PayloadSelection)
          }
        >
          <option value="text">テキスト</option>
          <option value="source-file">ソースファイル</option>
          <option value="project">プロジェクト</option>
        </select>
      </label>
      <label>
        <span>モード</span>
        <select
          value={mode}
          onChange={(event) =>
            onModeChange(event.target.value as ModeSelection)
          }
        >
          <option value="auto">おまかせ · MIDI</option>
          <option value="fast">高密度 · MIDI</option>
          <option value="music">音楽的 · MIDI</option>
          <option value="reliable">空気伝送 · WAV</option>
          <option value="secure">暗号化 · WAV</option>
        </select>
      </label>
      <p className="mode-hint">{modeDescription(mode)}</p>
      <div className="codec-protocol">{protocol}</div>
    </div>
  );
}
