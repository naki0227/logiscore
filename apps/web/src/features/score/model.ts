import { parseMidiNotes } from "../../hooks/useAudioEngine";

/** 五線譜に並べる音符の数（これより多い分は先頭だけを表示する） */
export const SCORE_NOTE_LIMIT = 24;

/** ハ長調の音階（C D E F G A B）を半音で表したもの */
const C_MAJOR = [0, 2, 4, 5, 7, 9, 11];

/**
 * 入力テキストのバイト列から、変換前の「音のイメージ」を作る。
 * 実際の符号化とは別物で、入力に合わせて譜面が動くことを見せるためのもの。
 */
export function previewPitches(
  text: string,
  limit = SCORE_NOTE_LIMIT,
): number[] {
  const bytes = new TextEncoder().encode(text);
  const pitches: number[] = [];
  for (const byte of bytes.slice(0, limit)) {
    const degree = byte % C_MAJOR.length;
    const octave = (byte >> 3) % 2;
    pitches.push(64 + C_MAJOR[degree] + octave * 12);
  }
  return pitches;
}

/** 変換したMIDIから、実際に鳴る音の高さを先頭から取り出す */
export function midiPitches(
  midi: Uint8Array,
  limit = SCORE_NOTE_LIMIT,
): number[] {
  try {
    return parseMidiNotes(midi)
      .notes.slice(0, limit)
      .map((note) => note.note);
  } catch {
    return [];
  }
}

/** 半音の位置 → 幹音（ドレミ…）の段数 */
const DIATONIC_INDEX = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];

/**
 * 音の高さを、五線の一番下の線（E4）から数えた段数に変える。
 * 線と間を合わせて 0〜8 の9段に収まるよう、オクターブを折り返す。
 */
export function staffStep(pitch: number): number {
  const absolute =
    Math.floor(pitch / 12) * 7 + DIATONIC_INDEX[((pitch % 12) + 12) % 12];
  const fromE4 = absolute - (5 * 7 + 2);
  return ((fromE4 % 9) + 9) % 9;
}
