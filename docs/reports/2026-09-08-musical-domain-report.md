# 作業報告書

## 作業日時

2026年09月08日 20時58分39秒 JST

## 作業対象

v2 Phase 2 Musical Baselineのdomain modelと可逆symbol codec。

## 作業目的

PayloadやMIDIへ依存せず、調性・和声・4声制約から決定的な16候補を生成し、4 bit symbolを完全可逆に選択できる基盤を作る。

## 変更内容

- Major、Natural Minor、Dorian、LydianのTonal Contextを追加した。
- 4 event周期の`I → V → vi → IV` Chord Plannerを追加した。
- Bass、Harmony 2、Harmony 1、Melodyの固定voice rangeを定義した。
- chord toneとstrict ascending制約から4声voicingを列挙するCandidate Generatorを追加した。
- 音域中心、共通音、stepwise motion、leap、consonance、voice spacingをscore化した。
- score降順とnote辞書順からcanonicalな16候補を生成した。
- `0..15`のsymbolとvoicingを相互変換するCandidate Set APIを追加した。
- chord進行と直前voicing stateを追跡するMusical Symbol Codecを追加した。
- Phase 1のnpm auditを再実行し、high/critical 0件を確認した。

## 変更したファイル

- `packages/harmonic-core/src/lib.rs`
- `packages/harmonic-core/src/musical/mod.rs`
- `packages/harmonic-core/src/musical/tonal.rs`
- `packages/harmonic-core/src/musical/harmony.rs`
- `packages/harmonic-core/src/musical/candidates.rs`
- `packages/harmonic-core/src/musical/codec.rs`
- `packages/harmonic-core/src/musical/tests.rs`
- `docs/adr/0003-musical-baseline.md`
- `docs/TODO.md`

## 変更意図

候補生成の決定性がcodecの互換性そのものになるため、MIDI parserやUIより先に純粋なdomainとして仕様とテストを固定した。

## 設計上の意図

Tonal Context、Chord Planner、候補生成、stream stateを別moduleへ分離した。すべての実装fileを300行以内に保ち、外部依存を追加していない。score weightは将来profile versionの一部として扱い、暗黙に変更しない。

## 影響範囲

Rust crateに新しいpublic `musical` moduleを追加する。既存Dense transport、v1/v2 packet、WASM、Web UIの挙動は変更しない。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- 不正root pitch classの拒否
- mode scaleの移調
- `I–V–vi–IV`の周期性
- 各chordで16個の一意かつ有効な候補生成
- previous voicing込みの決定性
- 同一chordでprevious voicingを最優先
- 全16 symbolのencode/decode
- 4声のstrict orderingとrange
- chord/stateをまたぐsymbol stream round-trip

## 実行した確認コマンド

- `npm audit --audit-level=high --fetch-timeout=10000 --fetch-retries=0`: 成功。high/critical 0件、moderate 2件、low 1件
- `cargo fmt --all`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 90件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run typecheck:e2e`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 19件成功
- `npm run build`: 成功。JS chunk約688 kBのwarningは継続
- `npm run test:e2e`: Chromium 4件成功
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、336 iterations、1,346 HTTP requests
  - checks 100%、request failure 0%、p95 10.5 ms
- `git diff --check`: 成功

## CIで確認される内容

Rust format、Clippy、unit test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- Musical Symbol CodecはまだSMF MIDIへserializeされない。
- 固定進行とchord toneのみのため音楽表現は限定的。
- score weight変更時のcodec profile versioningは未実装。
- DOMPurify/Monaco等のmoderate 2件・low 1件は残る。

## 次にやること

Musical Symbol CodecをMusical MIDI Transportへ接続し、v2 packetのcodec profile `1`としてText round-tripを実装する。その後WASM/Web Music modeへ公開する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/musical/codec.rs`
- `packages/harmonic-core/src/transport/mod.rs`
- `packages/harmonic-core/src/v2_packet.rs`
- `docs/adr/0003-musical-baseline.md`
- `docs/TODO.md`

## 引き継ぎ事項

候補のscore順はwire compatibilityに影響するため、weightやtie-breakerを無断で変更しない。Musical MIDI Transportはdomainの内部実装へMIDI知識を逆流させず、`MusicalSymbolCodec`のpublic APIだけを利用する。既存Dense経路とv1 decode fallbackは維持する。
