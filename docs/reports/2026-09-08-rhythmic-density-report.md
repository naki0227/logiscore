# 作業報告書

## 作業日時

2026年09月08日 21時29分02秒 JST

## 作業対象

v2 Phase 3 Musical Density、Rhythmic Musical MIDI profile `2`。

## 作業目的

PitchだけでなくRhythmへdataを分散し、動的候補数によってMusical codecのevent密度を改善しながら完全可逆性を維持する。

## 変更内容

- Candidate Setを2〜32のpower-of-two候補数へ拡張した。
- 強拍8候補、弱拍32候補の決定的Dynamic Candidate Countを追加した。
- Pitch 3/5 bitとRhythm 2 bitを組み合わせる可変容量bitstream codecを追加した。
- 240、360、480、720 ticksのduration候補を追加した。
- codec profile `2`のRhythmic MIDI writer/parserを追加した。
- 元packet長、0 padding、4声duration一致、event上限を検証した。
- Text、Source File、ProjectのRust/WASM encode/decodeを追加した。
- Web Music modeの新規encodeをprofile `2`へ切り替え、profile `1` decode互換を維持した。
- Playwrightの全Music modeシナリオをprofile `2`で自動実行した。

## 変更したファイル

- `packages/harmonic-core/src/musical/mod.rs`
- `packages/harmonic-core/src/musical/candidates.rs`
- `packages/harmonic-core/src/musical/rhythm.rs`
- `packages/harmonic-core/src/musical/tests.rs`
- `packages/harmonic-core/src/transport/mod.rs`
- `packages/harmonic-core/src/transport/rhythmic_midi.rs`
- `packages/harmonic-core/src/transport/rhythmic_midi/tests.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/v2/rhythmic.rs`
- `packages/harmonic-core/src/v2_packet.rs`
- `packages/harmonic-core/src/wasm_v2.rs`
- `apps/web/src/pkg/harmonic_core*`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/codec/model.test.ts`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `docs/adr/0005-rhythmic-musical-density.md`
- `README.md`
- `docs/TODO.md`

## 変更意図

Phase 2 profileの互換性を変えず、明示的な新profileとして密度改善を導入するため。

## 設計上の意図

bitstreamと候補選択はdomain、SMF表現はTransport、payload packetはv2 APIに分離した。Velocityへdataを格納せず、将来のAcoustic decodeを妨げない。全実装fileを300行以内に保ち、新規依存は追加していない。

## 影響範囲

Web Music modeで新規生成されるMIDIはprofile `2`になる。既存profile `1`、Dense、v1 MIDIのimport互換性は維持する。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- 8/16/32候補の生成と制約維持
- Pitch/Rhythm bitstreamの全byte値round-trip
- 空payloadとpadding
- profile `1`より少ないevent数・小さいMIDI size
- 複数durationの使用
- 4声SMFのencode/decode
- profile `2` header検証
- Text Unicode、Source metadata、Project files round-trip
- PlaywrightによるMusic Text／Source File／Project download/import

## 実行した確認コマンド

- `wasm-pack build . --target web --out-dir ../../apps/web/src/pkg`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 110件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run typecheck:e2e`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 22件成功
- `npm run build`: 成功。WASM約334 kB、JS約689 kB
- `npm run test:e2e`: Chromium 7件成功
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、330 iterations、1,322 HTTP requests
  - checks 100%、request failure 0%、p95 11.95 ms
- `npm audit --audit-level=high`: 成功。high/critical 0件、moderate 2件、low 1件
- `git diff --check`: 成功

## CIで確認される内容

Rust format、Clippy、test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- Audio recordingからのsymbol/duration検出は未実装。
- rest、syncopation、articulationは未使用。
- MIDI length framingはText metadataを利用している。
- JS bundleのcode splitting warningとmoderate/low依存脆弱性が残る。

## 次にやること

Phase 4としてPCM入力、周波数検出、timing recovery、preambleを実装し、生成音声からpacketを復元するintegration testを追加する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/musical/rhythm.rs`
- `packages/harmonic-core/src/transport/rhythmic_midi.rs`
- `docs/adr/0005-rhythmic-musical-density.md`
- `docs/logiscore_improvement_design.md`
- `docs/TODO.md`

## 引き継ぎ事項

profile `2`の5/7 bit容量、duration table、MSB-first、0 padding、length metadataはwire compatibilityである。変更時は新profileを使う。ブラウザ挙動変更時は必ずPlaywrightシナリオを追加・自動実行する。
