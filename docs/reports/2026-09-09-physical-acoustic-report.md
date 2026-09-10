# 作業報告書

## 作業日時

2026年09月09日 19時43分52秒 JST

## 作業対象

v2 Phase 6 Physical Acoustic、WAV/resampling/channel model、Reliable Web UI、録音file workflow。

## 作業目的

Reliable PCMを標準音声fileとして再生・共有・録音・再importできるようにし、異なるsample rate・channel数・browser codecを同じRust decoderへ接続する。

## 変更内容

- strict RIFF/WAVE parserと8 kHz mono PCM16 encoderを追加した。
- PCM16/float32、mono/stereo、1〜32 channels、最大384 kHzの入力検証とdownmixを追加した。
- Rust linear sample-rate normalizationを追加した。
- gain、noise、clipping、leading offset、echoを再現するChannelModelを追加した。
- Text、Source File、ProjectのReliable WAV Rust/WASM APIを追加した。
- M4A/MP3/Opus/Ogg/AACをWeb Audioでmono PCM化するbrowser境界を追加した。
- Reliable modeを有効化し、WAV生成・検証・再生・download・recording importを実装した。
- Playwrightへ全payload WAV往復、Web Audio PCM経路、破損録音拒否を追加した。
- Playwrightで発見したWASM 32-bit resample length overflowを`u64`計算とsame-rate fast pathで修正した。
- Vitest 4.1.11へ更新し、DOMPurify 3.4.15をoverrideしてnpm auditを0件にした。
- Web側READMEをproject固有の開発・検証・Reliable workflowへ更新した。

## 変更したファイル

- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/audio/channel.rs`
- `packages/harmonic-core/src/audio/resample.rs`
- `packages/harmonic-core/src/audio/wav.rs`
- `packages/harmonic-core/src/audio/wav/tests.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/v2/pcm.rs`
- `packages/harmonic-core/src/v2/wav.rs`
- `packages/harmonic-core/src/wasm_audio.rs`
- `packages/harmonic-core/src/lib.rs`
- `apps/web/src/lib/acoustic-codec.ts`
- `apps/web/src/hooks/useAcousticCodec.ts`
- `apps/web/src/hooks/useWavPlayback.ts`
- `apps/web/src/features/audio/decodeAudioFile.ts`
- `apps/web/src/features/audio/decodeAudioFile.test.ts`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/codec/model.test.ts`
- `apps/web/src/features/codec/CodecSelector.tsx`
- `apps/web/src/features/codec/createEncodingActions.ts`
- `apps/web/src/features/files/usePayloadImport.ts`
- `apps/web/src/features/workspace/ControlsPanel.tsx`
- `apps/web/src/features/workspace/OutputPanel.tsx`
- `apps/web/src/features/workspace/SourcePanel.tsx`
- `apps/web/src/App.tsx`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `apps/web/package.json`
- `apps/web/package-lock.json`
- `apps/web/README.md`
- `apps/web/src/pkg/harmonic_core*`
- `README.md`
- `docs/adr/0008-physical-audio-workflow.md`
- `docs/TODO.md`

## 変更意図

信号処理をRust coreへ集約しつつ、browserが既に持つ圧縮音声decoderを境界adapterとして利用し、外部録音workflowを依存追加なしで成立させるため。

## 設計上の意図

WAV parsing、resampling、channel simulation、WASM、browser file decode、playback、UI routingを責務別に分割した。Appは279行、新規production fileはすべて300行以内である。file/sample/channel/rateに上限を設け、decoder詳細はuser statusへ露出しない。

## 影響範囲

WebのReliable modeが利用可能になり、出力形式はWAVになる。Auto/FastのDense MIDI、MusicのRhythmic MIDI、v1/v2 import互換性は維持する。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- PCM16 quantization round-trip
- PCM16 stereo downmix、float32 mono decode
- unknown even/odd RIFF chunkとpadding
- malformed length・unsupported encoding拒否
- 8 kHz→48 kHz→8 kHz recording resampling
- combined gain/noise/clipping/offset/echoでReliable payload復元
- audio channel/file入力境界の異常系
- Web codec routeとPCM+FEC label
- PlaywrightによるText／Source File／Project WAV download/import
- PlaywrightによるWeb Audio decoded PCM import
- Playwrightによる破損録音の安全な拒否

## 実行した確認コマンド

- `wasm-pack build . --target web --out-dir ../../apps/web/src/pkg`: 成功
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 147件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run typecheck:e2e`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: Vitest 4.1.11、27件成功
- `npm run build`: 成功。WASM約387 kB、JS約697 kB
- `npm run test:e2e`: Chromium 14件成功
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、306 iterations、1,226 HTTP requests
  - checks 100%、request failure 0%、p95 14.35 ms
- `npm audit --audit-level=high`: 0 vulnerabilities
- `git diff --check`: 成功

## CIで確認される内容

Rust format、Clippy、test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- 実端末、距離、部屋、会話、TV等の測定結果はPhase 9 benchmarkで収集する。
- linear resamplerに高度なanti-alias filterはない。
- browser codec supportはOS/browserに依存する。
- 大規模Reliable WAVは同期WASM処理と再生時間が長い。
- JS bundleのcode splitting warningが残る。

## 次にやること

Phase 7としてEnvironment Profile、calibration signal、confidence score、fallback、duration予測を実装する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/audio/decode.rs`
- `packages/harmonic-core/src/audio/channel.rs`
- `apps/web/src/hooks/useAcousticCodec.ts`
- `docs/adr/0008-physical-audio-workflow.md`
- `docs/TODO.md`

## 引き継ぎ事項

8 kHz、preamble/header tones、WAV PCM16形式は互換性に関わる。変更時はaudio profileをversion化する。圧縮音声はWeb Audio対応範囲であり、WAVはRustがauthoritative decoderである。ブラウザ確認は必ずPlaywright scriptとして追加し自動実行する。

既存worktreeにはPhase 1〜6が未コミットで共有fileが重なる。コミット時はphase順のpatch stagingを行い、Vitest/DOMPurify更新は`chore(web): resolve dependency advisories`として機能変更と分ける。
