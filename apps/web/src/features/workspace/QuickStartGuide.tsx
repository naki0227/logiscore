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
      title: "書く",
      body: "メッセージを入力するか、コードやプロジェクトフォルダを読み込みます。",
    },
    {
      key: "encoded",
      title: "変換する",
      body: "「変換する」を押すと、音楽（MIDIまたはWAV）になります。",
    },
    {
      key: "played",
      title: "再生して録音",
      body: "「再生」で音を鳴らし、別の端末で録音します。ファイルとして保存しても構いません。",
    },
    {
      key: "restored",
      title: "復元する",
      body: "録音やファイルを読み込むと、元のデータに戻ります。",
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
        使い方を見る
      </button>
    );
  }

  const current = STEPS.findIndex((step) => !progress[step.key]);

  return (
    <section className="quickstart" aria-label="はじめに">
      <div className="quickstart-intro">
        <p className="quickstart-eyebrow">Code / Browser / Sound</p>
        <h1 className="quickstart-title">
          データを<em>音にして</em>送る
        </h1>
        <p className="quickstart-lead">
          テキスト、ソースコード、プロジェクトを音楽に変換します。スピーカーで鳴らして別の端末で録音し、その録音を読み込むと元のデータがそのまま戻ります。処理はすべてブラウザの中で行われ、データはどこにも送信されません。
        </p>
        <div className="quickstart-actions">
          <button className="btn quickstart-demo" onClick={onLoadDemo}>
            空気伝送のデモを試す
          </button>
          <button
            className="quickstart-dismiss"
            onClick={() => {
              writeDismissed(true);
              setDismissed(true);
            }}
          >
            ガイドを閉じる
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
