import { useState } from "react";

const STORAGE_KEY = "logiscore.quickstart.dismissed";

export interface QuickStartProgress {
  hasInput: boolean;
  encoded: boolean;
  played: boolean;
  restored: boolean;
}

interface QuickStartGuideProps {
  progress: QuickStartProgress;
  onLoadDemo: () => void;
}

const STEPS: { key: keyof QuickStartProgress; title: string; body: string }[] =
  [
    {
      key: "hasInput",
      title: "Write",
      body: "Type a message, paste code, or drop a project folder.",
    },
    {
      key: "encoded",
      title: "Encode",
      body: "Press ENCODE to turn it into music (MIDI or WAV).",
    },
    {
      key: "played",
      title: "Play & record",
      body: "Press PLAY out loud and record it on another device, or download the file.",
    },
    {
      key: "restored",
      title: "Restore",
      body: "Import the recording or file with the import button to get the original back.",
    },
  ];

function readDismissed(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

function writeDismissed(dismissed: boolean) {
  try {
    if (dismissed) window.localStorage.setItem(STORAGE_KEY, "1");
    else window.localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Storage may be unavailable (private mode); the guide still works.
  }
}

export function QuickStartGuide({
  progress,
  onLoadDemo,
}: QuickStartGuideProps) {
  const [dismissed, setDismissed] = useState(readDismissed);

  if (dismissed) {
    return (
      <button
        className="quickstart-reopen"
        onClick={() => {
          writeDismissed(false);
          setDismissed(false);
        }}
      >
        How does it work?
      </button>
    );
  }

  const current = STEPS.findIndex((step) => !progress[step.key]);

  return (
    <section className="quickstart" aria-label="Quick start">
      <div className="quickstart-intro">
        <h1 className="quickstart-title">Send data with sound</h1>
        <p className="quickstart-lead">
          Logiscore turns text, source code, or a whole project into music. Play
          it out loud, record it on another device, and import the recording to
          get the exact original back. Everything runs in your browser — nothing
          is uploaded.
        </p>
        <div className="quickstart-actions">
          <button className="btn quickstart-demo" onClick={onLoadDemo}>
            Try the over-the-air demo
          </button>
          <button
            className="quickstart-dismiss"
            onClick={() => {
              writeDismissed(true);
              setDismissed(true);
            }}
          >
            Hide guide
          </button>
        </div>
      </div>
      <ol className="quickstart-steps">
        {STEPS.map((step, index) => {
          const state = progress[step.key]
            ? "done"
            : index === current
              ? "current"
              : "todo";
          return (
            <li key={step.key} className={`quickstart-step ${state}`}>
              <span className="quickstart-step-number" aria-hidden="true">
                {state === "done" ? "✓" : index + 1}
              </span>
              <span className="quickstart-step-text">
                <strong>{step.title}</strong>
                <span>{step.body}</span>
              </span>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
