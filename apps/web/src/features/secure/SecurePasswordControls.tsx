import type { SecurePasswordState } from "./model";

interface SecurePasswordControlsProps {
  password: string;
  confirmation: string;
  validation: SecurePasswordState;
  onPasswordChange: (password: string) => void;
  onConfirmationChange: (confirmation: string) => void;
}

export function SecurePasswordControls(props: SecurePasswordControlsProps) {
  return (
    <fieldset className="secure-password-controls">
      <legend>暗号化パスワード</legend>
      <label>
        <span>パスワード</span>
        <input
          type="password"
          value={props.password}
          autoComplete="new-password"
          onChange={(event) => props.onPasswordChange(event.target.value)}
        />
      </label>
      <label>
        <span>パスワード（確認）</span>
        <input
          type="password"
          value={props.confirmation}
          autoComplete="new-password"
          onChange={(event) => props.onConfirmationChange(event.target.value)}
        />
      </label>
      <div
        className={props.validation.valid ? "secure-valid" : "secure-invalid"}
        aria-live="polite"
      >
        {props.validation.message} （{props.validation.byteLength}/1024 バイト）
      </div>
      <small>
        パスワードはこのタブのメモリにだけ保持され、保存も送信もされません。
      </small>
    </fieldset>
  );
}
