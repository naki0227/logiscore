import { useState, useCallback, useRef } from 'react';
import * as Tone from 'tone';

/** 再生設定 */
export interface PlaybackConfig {
  /** 各ノートの発音時間 (秒) */
  noteDuration: number;
  /** Tick 間の再生間隔 (秒) */
  tickInterval: number;
}

const DEFAULT_CONFIG: PlaybackConfig = {
  noteDuration: 0.15,
  tickInterval: 0.2,
};

interface NoteEvent {
  tick: number;
  channel: number;
  note: number;
  velocity: number;
  program?: number; // 楽器番号を追加
  fileName?: string;
}

/**
 * MIDI バイナリからノートイベントを抽出する（簡易パーサー）。
 * WASM 側の midly に依存せず、JS 側で最小限のパースを行う。
 */
function parseMidiNotes(midiBytes: Uint8Array): {
  notes: NoteEvent[];
  scaleId: number;
  rootKey: number;
  dataLength: number;
  bytesPerTick: number;
} {
  const view = new DataView(midiBytes.buffer, midiBytes.byteOffset, midiBytes.byteLength);

  // MThd ヘッダーをスキップ (14 bytes)
  let pos = 14;

  // MTrk
  pos += 4; // "MTrk"
  const trackLen = view.getUint32(pos);
  pos += 4;
  const trackEnd = pos + trackLen;

  const notes: NoteEvent[] = [];
  let absTick = 0;
  let scaleId = 0;
  let rootKey = 0;
  let dataLength = 0;
  let bytesPerTick = 8;
  let runningStatus = 0;
  let currentFileName = '';
  const channelPrograms: number[] = new Array(16).fill(0); // チャンネルごとの現在音色

  function readVlq(): number {
    let value = 0;
    while (pos < trackEnd) {
      const byte = midiBytes[pos++];
      value = (value << 7) | (byte & 0x7F);
      if ((byte & 0x80) === 0) break;
    }
    return value;
  }

  while (pos < trackEnd) {
    const delta = readVlq();
    absTick += delta;

    // Running Status の処理
    let statusByte = midiBytes[pos];
    if (statusByte < 0x80) {
      statusByte = runningStatus;
    } else {
      runningStatus = statusByte;
      pos++;
    }

    if (statusByte === 0xFF) {
      // Meta event
      const metaType = midiBytes[pos++];
      const metaLen = readVlq();
      const metaData = midiBytes.slice(pos, pos + metaLen);
      pos += metaLen;

      if (metaType === 0x01) {
        // Text event
        const text = new TextDecoder().decode(metaData);
        if (text.startsWith('SCALE:')) scaleId = parseInt(text.slice(6));
        if (text.startsWith('ROOT:')) rootKey = parseInt(text.slice(5));
        if (text.startsWith('BPT:')) bytesPerTick = parseInt(text.slice(4));
        if (text.startsWith('LEN:')) dataLength = parseInt(text.slice(4));
      } else if (metaType === 0x06) {
        // Marker event
        const text = new TextDecoder().decode(metaData);
        if (text.startsWith('FILE:')) {
            currentFileName = text.slice(5);
        }
      }
    } else if ((statusByte & 0xF0) >= 0x80) {
      // MIDI event
      const type = statusByte & 0xF0;
      const channel = statusByte & 0x0F;

      if (type === 0x90) {
        // NoteOn
        const key = midiBytes[pos++];
        const vel = midiBytes[pos++];
        if (vel > 0) {
          notes.push({ 
              tick: absTick, 
              channel, 
              note: key, 
              velocity: vel, 
              program: channelPrograms[channel] || 0, // 直近のプログラムを適用
              fileName: currentFileName 
          });
        }
      } else if (type === 0x80) {
        // NoteOff
        pos += 2;
      } else if (type === 0xC0) {
        // Program Change: チャンネルごとの音色を記録
        const program = midiBytes[pos++];
        channelPrograms[channel] = program;
      } else if (type === 0xD0) {
        pos += 1;
      } else {
        pos += 2;
      }
    }
  }

  return { notes, scaleId, rootKey, dataLength, bytesPerTick };
}

export function useAudioEngine() {
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(0);
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const stopRef = useRef(false);

  /** 再生を開始 */
  const play = useCallback(async (
    midiBytes: Uint8Array,
    config: PlaybackConfig = DEFAULT_CONFIG,
    onNote?: (note: number, velocity: number, duration: number, isBass: boolean) => void,
    onFileChange?: (fileName: string) => void,
    onComplete?: () => void
  ) => {
    await Tone.start();

    // オーケストラ用シンセのセットアップ
    const synths: Record<string, Tone.PolySynth> = {};
    const delay = new Tone.PingPongDelay("8n", 0.2).toDestination();
    const reverb = new Tone.Reverb(2.5).toDestination();

    const getSynth = (program: number) => {
        const type = 
            program === 48 ? 'strings' :
            program === 42 || program === 45 ? 'cello' :
            program === 60 || program === 56 ? 'brass' :
            program === 71 || program === 73 ? 'woodwind' :
            program === 46 || program === 10 ? 'keyboard' :
            program === 47 || program === 14 ? 'percussion' : 'piano';
        
        if (synths[type]) return synths[type];

        let synth: Tone.PolySynth;
        if (type === 'strings') {
            synth = new Tone.PolySynth(Tone.Synth, {
                oscillator: { type: 'sawtooth' },
                envelope: { attack: 0.2, decay: 0.1, sustain: 1, release: 1 }
            });
        } else if (type === 'brass') {
            synth = new Tone.PolySynth(Tone.Synth, {
                oscillator: { type: 'sawtooth' },
                envelope: { attack: 0.05, decay: 0.2, sustain: 0.4, release: 0.1 }
            });
        } else if (type === 'woodwind') {
            synth = new Tone.PolySynth(Tone.Synth, {
                oscillator: { type: 'sine' },
                envelope: { attack: 0.1, decay: 0.2, sustain: 0.8, release: 0.2 }
            });
        } else if (type === 'percussion') {
            synth = new Tone.PolySynth(Tone.Synth, {
                oscillator: { type: 'square' },
                envelope: { attack: 0.001, decay: 0.5, sustain: 0, release: 0.1 }
            });
        } else {
            synth = new Tone.PolySynth(Tone.Synth, {
                oscillator: { type: 'triangle' },
                envelope: { attack: 0.005, decay: 0.1, sustain: 0.3, release: 1 }
            });
        }

        synth.connect(delay);
        synth.connect(reverb);
        synth.toDestination();
        synths[type] = synth;
        return synth;
    };

    const { notes } = parseMidiNotes(midiBytes);

    // Tick ごとにグループ化
    const tickMap = new Map<number, NoteEvent[]>();
    for (const note of notes) {
      if (!tickMap.has(note.tick)) tickMap.set(note.tick, []);
      tickMap.get(note.tick)!.push(note);
    }
    const ticks = [...tickMap.keys()].sort((a, b) => a - b);

    // 全体の Tick 数から最適な間隔を算出
    // 目標時間: 約 45s
    const targetSeconds = 45;
    const numTicks = ticks.length;
    let autoTickInterval = config.tickInterval || 0.25;
    
    if (numTicks > 0) {
        const ideal = targetSeconds / numTicks;
        // 0.1s (BPM 600) 〜 0.8s (BPM 75) の範囲で固定
        autoTickInterval = Math.max(0.1, Math.min(0.8, ideal));
    }

    stopRef.current = false;
    setPlaying(true);
    setProgress(0);

    let lastUpdateTick = 0;
    let currentInLoopActiveFile = '';

    for (let i = 0; i < ticks.length; i++) {
      if (stopRef.current) break;

      const tickNotes = tickMap.get(ticks[i])!;

      for (let j = 0; j < tickNotes.length; j++) {
        const n = tickNotes[j];
        const isBass = j === 0;
        const finalMidiNote = isBass ? n.note - 24 : n.note;
        const noteName = Tone.Frequency(finalMidiNote, 'midi').toNote();
        const vel = n.velocity / 127;
        
        const strumDelay = isBass ? 0 : j * 0.015; 
        const duration = isBass ? autoTickInterval * 0.8 : autoTickInterval * 0.6;
        
        // プログラム番号に応じたシンセを選択
        const targetSynth = getSynth(n.program || 0);
        targetSynth.triggerAttackRelease(
          noteName,
          duration,
          Tone.now() + strumDelay,
          isBass ? vel * 0.4 : vel * 0.2
        );
        
        if (onNote) {
            setTimeout(() => onNote(finalMidiNote, vel, duration, isBass), strumDelay * 1000);
        }

        // ステート更新の最適化: ローカル変数で管理し、実際に変わる時だけ呼ぶ
        if (n.fileName && n.fileName !== currentInLoopActiveFile) {
            currentInLoopActiveFile = n.fileName;
            setActiveFile(n.fileName);
            onFileChange?.(n.fileName);
        }
      }

      // 進捗更新を間引く (100msごとに更新)
      const now = Date.now();
      if (now - lastUpdateTick > 100 || i === ticks.length - 1) {
          setProgress(((i + 1) / ticks.length) * 100);
          lastUpdateTick = now;
      }

      await new Promise(resolve => setTimeout(resolve, autoTickInterval * 1000));
    }

    setPlaying(false);
    setProgress(100);
    delay.dispose();
    reverb.dispose();
    // シンセの解放
    Object.values(synths).forEach((s: Tone.PolySynth) => s.dispose());
    onComplete?.();
  }, []);

  /** 再生を停止 */
  const stop = useCallback(() => {
    stopRef.current = true;
    setPlaying(false);
  }, []);

  return { playing, progress, activeFile, play, stop };
}
