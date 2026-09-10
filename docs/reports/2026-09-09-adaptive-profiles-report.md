# 作業報告書

## 作業日時

2026年09月09日 20時25分55秒 JST

## 作業対象

v2 Phase 7 Adaptive Profiles（Rust domain/audio/FEC、WASM、React UI、Playwright）。

## 作業目的

環境と優先度に応じて実際の音響signalを変更し、受信側がcalibrationとfallbackで自動復号できる状態にする。

## 変更内容

- Quiet / Balanced / Conversation / Noisy / Online / Long Distance / Fixed Fallbackを固定wire ID化した。
- timing 75〜200%、FEC 0〜3、interleave depth 8/16、repetition 1/3/5をprofileごとに実適用した。
- noise floor、SNR、clipping、末尾減衰のcalibration近似と決定的selectorを実装した。
- profile候補の自動再試行、旧flags 0 Balanced WAV互換、単声FixedFallbackを実装した。
- Environment、Reliability Priority、profile、confidence、Estimated DurationをReliable UIへ追加した。
- WASM境界と全Environmentの自動Playwright round-tripを追加した。
- PCM出力を16,000,000 samples上限でencode前に検証した。

## 変更したファイル

- `packages/harmonic-core/src/adaptive/`
- `packages/harmonic-core/src/audio/profile.rs`, `audio/fixed.rs`, `audio/mod.rs`, `audio/synth.rs`, `audio/decode.rs`
- `packages/harmonic-core/src/error_correction/mod.rs`, `error_correction/tests.rs`
- `packages/harmonic-core/src/v2/adaptive_audio.rs`, `v2/adaptive_decode.rs`, `v2_packet.rs`
- `packages/harmonic-core/src/wasm_adaptive.rs`, `wasm_audio.rs`, `lib.rs`
- `apps/web/src/features/acoustic/`, `hooks/useAdaptiveProfile.ts`, `hooks/useAcousticCodec.ts`
- `apps/web/src/lib/acoustic-codec.ts`, `features/workspace/ControlsPanel.tsx`, `features/codec/createEncodingActions.ts`, `App.tsx`, `App.css`
- `apps/web/e2e/codec-roundtrip.spec.ts`, generated WASM package
- `docs/adr/0009-adaptive-profile-selection.md`, `docs/TODO.md`

## 変更意図

Environmentを見た目だけの設定にせず、通信時間・冗長度・復号経路へ反映するため。profile ID検証により、誤ったtiming/FECで偶然復号したデータを受理しない。

## 設計上の意図

selector、signal codec、WASM、UIを分離し、決定的なtableを単一のRust domainに置いた。adaptive encode/decodeは182/181行へ分割し、既存Reliable APIは互換境界として維持した。新規依存は追加していない。

## 影響範囲

Reliable WAV生成・WAV/recorded PCM復号・profile表示。Dense/Musical MIDIと旧Reliable WAVの公開挙動は維持する。DB・認証・外部API変更はない。

## 追加・更新したテスト

- Rust: calibration境界、selector、duration、全7 profile WAV round-trip、旧WAV互換、Fixed codec、FEC 2/3、5-copy damage recovery。
- Vitest: WASM JSON profile schemaとduration表示。
- Playwright: Environment/Priority UI、選択可能な6環境のWASM WAV round-trip。既存download/import/recorded PCMも全再実行。

## 実行した確認コマンド

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features --locked -- -D warnings` 成功
- `cargo test --all-targets --all-features --locked` 162件成功
- rustup toolchainを明示した `wasm-pack build --target web --out-dir ../../apps/web/src/pkg --no-opt` 成功
- `npm run typecheck`, `typecheck:e2e`, `format:check`, `lint`, `test`, `build` 成功（Vitest 30件）
- `npm run test:e2e` 成功（Chromium 16/16）
- `npm audit --audit-level=high` 0件
- k6: 5 VU、329 iterations、1,318 requests、checks 100%、failure 0%、p95 12.37ms
- `git diff --check` 成功

## CIで確認される内容

Rust fmt/clippy/test/check/build、Web typecheck/E2E typecheck/format/lint/unit/build、Playwright Chromium、k6、npm high監査。

## 未解決の課題

- Phase 8 Secure Mode、Phase 9 Benchmark。
- JS bundle 701KBのcode splitting。
- 既存`useAudioEngine.ts`とv1 protocol 2ファイルの300行超過。
- calibration thresholdはPhase 9実機測定で再評価が必要。

## 次にやること

Phase 8の暗号envelope、Argon2id、AEAD、password UX、authentication failure testを設計・実装する。

## 次回最初に見るべきファイル

`docs/logiscore_improvement_design.md` Secure Mode節、`packages/harmonic-core/src/v2_packet.rs`、`packages/harmonic-core/Cargo.toml`、`docs/TODO.md`。

## 引き継ぎ事項

profile IDはBinary Header flags、Fixedはcodec profile 3。flags 0は旧Reliable互換専用。WASM buildはPATHと`DYLD_LIBRARY_PATH`をrustup stableへ明示し、環境のwasm-opt権限問題を避けるため`--no-opt`が必要。ブラウザ確認は必ずPlaywrightスクリプト化して自動実行する。Phase単位の差分が共有ファイルで重なるため、コミット時はpatch stagingで論理単位を分ける。
