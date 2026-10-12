import {
  formatDuration,
  profileLabel,
  type AcousticEnvironment,
  type AcousticProfileDescription,
} from "./model";

interface AcousticProfileControlsProps {
  environment: AcousticEnvironment;
  reliabilityPriority: number;
  profile: AcousticProfileDescription | null;
  onEnvironmentChange: (environment: AcousticEnvironment) => void;
  onPriorityChange: (priority: number) => void;
}

export function AcousticProfileControls(props: AcousticProfileControlsProps) {
  return (
    <div className="acoustic-profile" aria-label="音響プロファイル設定">
      <label>
        <span>環境</span>
        <select
          aria-label="環境"
          value={props.environment}
          onChange={(event) =>
            props.onEnvironmentChange(event.target.value as AcousticEnvironment)
          }
        >
          <option value="auto">おまかせ</option>
          <option value="quiet">静かな部屋</option>
          <option value="conversation">会話がある</option>
          <option value="noisy">騒がしい</option>
          <option value="online">オンライン通話</option>
          <option value="long-distance">離れた距離</option>
        </select>
      </label>
      <label className="priority-control">
        <span>速さ ↔ 確実さ</span>
        <input
          aria-label="信頼性の優先度"
          type="range"
          min="0"
          max="100"
          value={props.reliabilityPriority}
          onChange={(event) =>
            props.onPriorityChange(Number(event.target.value))
          }
        />
        <div className="priority-value" aria-label="信頼性の優先度の値">
          確実さ {props.reliabilityPriority}%
        </div>
      </label>
      <div className="profile-summary" aria-live="polite">
        <ProfileValue
          label="プロファイル"
          value={props.profile ? profileLabel(props.profile) : "—"}
        />
        <ProfileValue
          label="推定の確かさ"
          value={props.profile ? `${props.profile.confidence_percent}%` : "—"}
        />
        <ProfileValue
          label="予想再生時間"
          value={formatDuration(props.profile?.estimated_duration_ms ?? NaN)}
        />
      </div>
    </div>
  );
}

function ProfileValue({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
