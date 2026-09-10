# 作業報告書

## 作業日時

2026年09月08日 21時15分00秒 JST

## 作業対象

v2 Musical MIDI Transport、WASM/Web Music mode、Playwright E2E。

## 作業目的

Musical domain codecを実際のMIDIへ接続し、Text、Source File、ProjectをWebブラウザから完全可逆に利用できるPhase 2を完成させる。

## 変更内容

- 4声chordをType 0 SMFへserialize/parseする`MusicalMidiTransport`を追加した。
- v2 codec profile `1`をMusicalとしてpacket encode/decodeへ追加した。
- Text、Source File、ProjectのMusical Rust APIを追加した。
- WASM encode APIを追加し、decodeはDense/Musicalを自動判別するようにした。
- WebのMusic modeを有効化し、全payloadをMusical routeへ接続した。
- Musical profile表示を`V2 / MUSICAL`へ変更した。
- Playwrightに全payloadのMusical download/import round-tripを追加した。
- 64 KiB packet上限と131,072 symbol上限を追加した。

## 変更したファイル

- `packages/harmonic-core/src/transport/mod.rs`
- `packages/harmonic-core/src/transport/musical_midi.rs`
- `packages/harmonic-core/src/musical/candidates.rs`
- `packages/harmonic-core/src/musical/tonal.rs`
- `packages/harmonic-core/src/v2_packet.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/wasm_v2.rs`
- `apps/web/src/pkg/harmonic_core*`
- `apps/web/src/lib/wasm-loader.ts`
- `apps/web/src/hooks/useEntropy.ts`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/codec/model.test.ts`
- `apps/web/src/features/codec/CodecSelector.tsx`
- `apps/web/src/features/workspace/ControlsPanel.tsx`
- `apps/web/src/App.tsx`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `docs/adr/0004-musical-midi-transport.md`
- `README.md`
- `docs/TODO.md`

## 変更意図

Phase 2の音楽理論をUI上で利用可能なcodecへ完成させつつ、Denseおよびv1互換経路を維持するため。

## 設計上の意図

Musical domainはMIDIを知らず、TransportがSMF表現だけを担当する。packet profile検証、MIDI marker、4声完全性、入力上限を境界で検証する。全実装fileを300行以内に保ち、新規依存は追加していない。

## 影響範囲

WebでMusic modeが選択可能になる。Text、Source File、Projectのencode形式が選択に応じてDenseまたはMusicalになる。importは両形式と既存v1を継続して読める。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- 全byte値のMusical transport round-trip
- SMF validityと決定的encode
- Velocity非依存
- packet/symbol上限
- codec profile `1`検証
- Text Unicode、Source metadata、Project filesのRust round-trip
- Dense/Musical transport取り違え拒否
- Web routingとprofile label
- PlaywrightによるText、Source File、Project Musical MIDI download/import round-trip

## 実行した確認コマンド

- `wasm-pack build . --target web --out-dir ../../apps/web/src/pkg`: 成功
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 99件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run typecheck:e2e`: 成功
- `npm run format:check`: 成功
- `npm run lint`: 成功
- `npm test`: 22件成功
- `npm run build`: 成功。JS約689 kBのchunk warningは継続
- `npm run test:e2e`: Chromium 7件成功
- `npm audit --audit-level=high`: 成功。high/critical 0件、moderate 2件、low 1件
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、328 iterations、1,314 HTTP requests
  - checks 100%、request failure 0%、p95 12.3 ms

## CIで確認される内容

Rust format、Clippy、test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- Musical MIDIはDenseより低密度で、長いpayloadの再生時間が長い。
- 録音音声からのdecodeはPhase 4以降。
- JS bundle約689 kBのcode splitting warningが残る。
- moderate 2件・low 1件の依存脆弱性が残る。

## 次にやること

Phase 3でRhythm Encoding、Multiple Symbol Channels、Dynamic Candidate Count、bitrate最適化を設計・実装する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/transport/musical_midi.rs`
- `packages/harmonic-core/src/musical/codec.rs`
- `docs/adr/0003-musical-baseline.md`
- `docs/adr/0004-musical-midi-transport.md`
- `docs/TODO.md`

## 引き継ぎ事項

codec profile `1`、候補score順、track marker、固定timingはwire compatibilityである。変更時は新profileを検討する。Playwrightシナリオを更新せずブラウザ挙動を変更しない。大きいpayloadにはDenseを利用し、Musical上限を無断で緩和しない。
