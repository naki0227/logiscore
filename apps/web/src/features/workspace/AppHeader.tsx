import type { StatusMessage } from "../status/model";

interface AppHeaderProps {
  status: StatusMessage;
  filename: string;
  projectFileCount: number | null;
}

export function AppHeader({
  status,
  filename,
  projectFileCount,
}: AppHeaderProps) {
  return (
    <header className="header">
      <div className="logo">
        <span className="logo-mark" aria-hidden="true">
          <span />
          <span />
          <span />
          <span />
        </span>
        <span className="logo-text">Logiscore</span>
      </div>
      <p
        className={`status-bar status-${status.tone}`}
        role="status"
        aria-live="polite"
      >
        {status.text}
      </p>
      <div className="header-meta">
        <span className="filename-badge">
          {filename}
          {projectFileCount !== null && `（${projectFileCount} ファイル）`}
        </span>
        <a
          className="studio-link"
          href="https://enludus.com"
          target="_blank"
          rel="noopener noreferrer"
        >
          Enludus
        </a>
      </div>
    </header>
  );
}
