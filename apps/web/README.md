# Logiscore Web

React、TypeScript、ViteとRust/WASMで動くLogiscoreのブラウザUIです。すべてのcodec処理はクライアント内で完結し、入力したsourceやprojectは外部へ送信されません。

## Modes

- `Auto` / `Fast`: v2 Dense MIDI
- `Music`: v2 Rhythmic Musical MIDI
- `Reliable`: CRC/FEC付きの8 kHz mono PCM16 WAV

Reliable modeではWAVを再生・downloadし、外部レコーダーで録音したfileを再importできます。PCM16/float32 WAVはRustで直接検証します。M4A、MP3、Opus、Ogg、AACはbrowserが対応している場合にWeb Audioでmono PCMへ変換します。

## Development

先にRust/WASMを生成してからWeb appを起動します。

```bash
cd ../../packages/harmonic-core
wasm-pack build . --target web --out-dir ../../apps/web/src/pkg

cd ../../apps/web
npm ci
npm run dev
```

## Quality gates

```bash
npm run typecheck
npm run typecheck:e2e
npm run format:check
npm run lint
npm test
npm run build
npm run test:e2e
npm audit --audit-level=high
```

PlaywrightはDense／Musical MIDIとReliable WAVについて、Text、Source File、Projectのdownload/import round-tripを実ブラウザで自動実行します。またWeb Audio経由の録音PCMと破損音声の拒否も検証します。

k6はproduction previewを対象に実行します。

```bash
npm run preview -- --host 0.0.0.0 --port 4173

docker run --rm \
  -v "$PWD/tests/load:/scripts:ro" \
  grafana/k6:2.2.0 run \
  -e BASE_URL=http://host.docker.internal:4173 \
  /scripts/static-assets.js
```
