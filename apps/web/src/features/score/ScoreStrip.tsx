import type { CSSProperties } from "react";
import { staffStep } from "./model";

interface ScoreStripProps {
  pitches: number[];
  caption: string;
  playing: boolean;
  /** 0〜100 */
  progress: number;
  /** 値が変わるたびに、音符が順に波打つ（復元に成功したときの合図） */
  celebrateKey: number;
}

const WIDTH = 520;
const HEIGHT = 124;
const BOTTOM_LINE = 84;
const LINE_GAP = 12;
const FIRST_X = 40;

function noteY(step: number) {
  return BOTTOM_LINE - step * (LINE_GAP / 2);
}

function toneClass(step: number) {
  if (step <= 2) return "tone-low";
  if (step <= 5) return "tone-mid";
  return "tone-high";
}

export function ScoreStrip({
  pitches,
  caption,
  playing,
  progress,
  celebrateKey,
}: ScoreStripProps) {
  const spacing =
    pitches.length > 0
      ? Math.min(26, (WIDTH - FIRST_X - 24) / pitches.length)
      : 26;
  const active = playing
    ? Math.min(
        pitches.length - 1,
        Math.floor((progress / 100) * pitches.length),
      )
    : -1;
  const playheadX = FIRST_X - 10 + (progress / 100) * pitches.length * spacing;

  return (
    <figure className="score">
      <svg
        className="score-svg"
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        role="img"
        aria-label={caption}
      >
        {[0, 1, 2, 3, 4].map((line) => (
          <line
            key={line}
            className="score-line"
            x1={12}
            x2={WIDTH - 12}
            y1={BOTTOM_LINE - line * LINE_GAP}
            y2={BOTTOM_LINE - line * LINE_GAP}
          />
        ))}
        <line
          className="score-bar"
          x1={12}
          x2={12}
          y1={BOTTOM_LINE - 4 * LINE_GAP}
          y2={BOTTOM_LINE}
        />
        <g key={celebrateKey} className={celebrateKey > 0 ? "score-wave" : ""}>
          {pitches.map((pitch, index) => {
            const step = staffStep(pitch);
            const x = FIRST_X + index * spacing;
            const y = noteY(step);
            const stemUp = step < 4;
            const state =
              index === active
                ? "is-active"
                : playing && index > active
                  ? "is-waiting"
                  : "";
            return (
              <g
                key={`${index}-${pitch}`}
                className={`score-note ${toneClass(step)} ${state}`}
                style={{ "--i": index } as CSSProperties}
              >
                <line
                  className="score-stem"
                  x1={stemUp ? x + 6.6 : x - 6.6}
                  x2={stemUp ? x + 6.6 : x - 6.6}
                  y1={y}
                  y2={stemUp ? y - 30 : y + 30}
                />
                <ellipse
                  cx={x}
                  cy={y}
                  rx={7.4}
                  ry={5.2}
                  transform={`rotate(-20 ${x} ${y})`}
                />
              </g>
            );
          })}
        </g>
        {playing && (
          <line
            className="score-playhead"
            x1={playheadX}
            x2={playheadX}
            y1={BOTTOM_LINE - 4 * LINE_GAP - 14}
            y2={BOTTOM_LINE + 14}
          />
        )}
      </svg>
      <figcaption className="score-caption">
        {pitches.length > 0 ? caption : "入力すると、ここに音符が並びます"}
      </figcaption>
    </figure>
  );
}
