import {
  formatDuration,
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
    <div className="acoustic-profile" aria-label="Acoustic profile settings">
      <label>
        <span>Environment</span>
        <select
          aria-label="Environment"
          value={props.environment}
          onChange={(event) =>
            props.onEnvironmentChange(event.target.value as AcousticEnvironment)
          }
        >
          <option value="auto">Auto</option>
          <option value="quiet">Quiet</option>
          <option value="conversation">Conversation</option>
          <option value="noisy">Noisy</option>
          <option value="online">Online</option>
          <option value="long-distance">Long Distance</option>
        </select>
      </label>
      <label className="priority-control">
        <span>Priority</span>
        <input
          aria-label="Reliability priority"
          type="range"
          min="0"
          max="100"
          value={props.reliabilityPriority}
          onChange={(event) =>
            props.onPriorityChange(Number(event.target.value))
          }
        />
        <div className="priority-value" aria-label="Reliability priority value">
          {props.reliabilityPriority}% reliability
        </div>
      </label>
      <div className="profile-summary" aria-live="polite">
        <ProfileValue label="Profile" value={props.profile?.name ?? "—"} />
        <ProfileValue
          label="Confidence"
          value={props.profile ? `${props.profile.confidence_percent}%` : "—"}
        />
        <ProfileValue
          label="Estimated Duration"
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
