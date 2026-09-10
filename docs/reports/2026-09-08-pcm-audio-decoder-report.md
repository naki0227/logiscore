# 作業報告書

## 作業日時

2026年09月08日 21時50分06秒 JST

## 作業対象

v2 Phase 4 Audio Decoder、PCM/WASM境界、Playwrightブラウザシナリオ、k6ローカル実行経路。

## 作業目的

Phase 3のRhythmic Musical symbolsをclean PCMへ変換し、開始位置・周波数・durationを検出して全v2 payloadを完全復元できる基準実装を作る。

## 変更内容

- 8 kHz mono PCM profileとsine-wave synthesizerを追加した。
- two-tone preamble、32-bit packet length、guard silenceを追加した。
- onset、tone end、duration、Goertzel周波数検出を追加した。
- 非有限値、truncation、同期不一致、packet/sample上限を検証した。
- v2 packet build/decodeをtransportから分離し、PCMとMIDIで共有した。
- Text、Source File、ProjectのRust/WASM/TypeScript PCM APIを追加した。
- Playwrightで全PCM payloadを実ブラウザ内WASM round-tripした。
- Docker上のk6からVite previewへ接続するhostを限定許可した。

## 変更したファイル

- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/audio/profile.rs`
- `packages/harmonic-core/src/audio/synth.rs`
- `packages/harmonic-core/src/audio/decode.rs`
- `packages/harmonic-core/src/audio/tests.rs`
- `packages/harmonic-core/src/musical/codec.rs`
- `packages/harmonic-core/src/v2_packet.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/v2/pcm.rs`
- `packages/harmonic-core/src/wasm_v2.rs`
- `packages/harmonic-core/src/error.rs`
- `packages/harmonic-core/src/lib.rs`
- `apps/web/src/lib/wasm-loader.ts`
- `apps/web/src/pkg/harmonic_core*`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `apps/web/vite.config.ts`
- `README.md`
- `docs/adr/0006-pcm-audio-decoder.md`
- `docs/TODO.md`

## 変更意図

実録音へ進む前に、transport非依存packetと信号検出の境界を確立し、誤り訂正や物理試験の失敗原因を分離できるようにするため。

## 設計上の意図

対象周波数が既知のためGoertzelを採用し、FFT依存は追加していない。musical candidate生成を再利用し、audio層はPCM framingと検出だけを担当する。全新規実装ファイルは300行以内で、外部入力にはサイズ・有限値・構造検証を行う。

## 影響範囲

新しいPCM APIと生成WASMが追加される。既存Dense、Musical profile 1、Rhythmic profile 2、v1 MIDIの挙動とimport互換性は変更しない。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- clean PCM packet round-trip
- leading sample offsetからのtiming recovery
- low deterministic noise耐性
- preamble欠落・NaN・上限超過拒否
- Text Unicode、Source metadata、Project archive PCM round-trip
- payload type不一致拒否
- Playwright Chromiumによる全payloadのPCM/WASM round-trip

## 実行した確認コマンド

- `wasm-pack build . --target web --out-dir ../../apps/web/src/pkg`: 成功
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 119件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run typecheck:e2e`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 22件成功
- `npm run build`: 成功。WASM約357 kB、JS約689 kB
- `npm run test:e2e`: Chromium 8件成功
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、314 iterations、1,258 HTTP requests
  - checks 100%、request failure 0%、p95 11.69 ms
  - 初回はViteが`host.docker.internal`を403拒否したため、任意hostを開放せず当該hostだけを許可して再実行した
- `npm audit --audit-level=high`: 成功。high/critical 0件、moderate 2件、low 1件

## CIで確認される内容

Rust format、Clippy、test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- CRC、FEC、Interleaving、repetitionはPhase 5。
- resampling、gain variation、clipping、room response、実録音はPhase 6以降。
- JS bundleのcode splitting warningとmoderate/low依存脆弱性が残る。
- PCM APIはWASMまで公開済みだが、録音・WAV操作UIは未実装。

## 次にやること

Phase 5としてpacket integrity、FEC framing、interleaving、repetitionを設計・実装する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/v2_packet.rs`
- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/protocol/v2.rs`
- `docs/adr/0006-pcm-audio-decoder.md`
- `docs/TODO.md`

## 引き継ぎ事項

PCM framing、preamble notes、header notes、sample rateは互換性に関わる。変更時はaudio profileをversion化する。ブラウザ確認は必ずPlaywrightスクリプトとして追加し、自動実行する。k6をDockerでローカル実行するときはpreviewを`--host 0.0.0.0`で起動する。

既存worktreeにはPhase 1〜4が未コミットで、`v2.rs`、`wasm_v2.rs`、生成WASM、README/TODOなどがphase間で重なっている。履歴を偽って混在コミットにせず、コミット時は `ci/test基盤`、`Phase 1 protocol/payload`、`Phase 2 musical`、`Phase 3 rhythmic`、`Phase 4 PCM` の順にpatch stagingして各実装と対応テスト・ADRを同じコミットへ含める。
