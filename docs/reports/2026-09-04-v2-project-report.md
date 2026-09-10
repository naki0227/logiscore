# 作業報告書

## 作業日時

2026年09月04日 17時27分59秒 JST

## 作業対象

v2 Project canonical archive、Rust/WASM/Web接続、Playwrightシナリオテスト。

## 作業目的

複数のUTF-8 source fileを決定的かつ安全なv2 payloadとしてDense MIDIへ変換し、ブラウザでMIDI再取込まで保証する。

## 変更内容

- Project headerと長さ付きfile entryからなるcanonical binary schemaを追加した。
- pathをUTF-8 bytes順に整列し、入力順に依存しないencodeにした。
- path traversal、重複path、不正文字、切断、余剰data、未対応version/encodingを拒否した。
- 2,048 files、path 1,024 bytes、segment 255 bytes、1 file 8 MiB、source合計32 MiBの上限を追加した。
- symlinkやdirectory metadataをschema対象外とし、regular UTF-8 source fileだけを格納する設計にした。
- v2 packet処理を共通moduleへ分離し、Text、Source File、Projectで再利用した。
- Project encode/decodeをRust、WASM、Webへ公開し、WebのProject経路をv1からv2 Denseへ切り替えた。
- import時はv2 Projectを先に判定し、既存v1 Projectのdecode fallbackを維持した。
- Project画面のフォルダ選択とMIDI選択を別controlへ分離した。
- Project fixture 2 filesを使い、フォルダ取込、MIDI download、再import、path復元をPlaywrightで検証した。

## 変更したファイル

- `packages/harmonic-core/src/project_payload.rs`
- `packages/harmonic-core/src/project_payload/tests.rs`
- `packages/harmonic-core/src/payload.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/v2_packet.rs`
- `packages/harmonic-core/src/wasm_v2.rs`
- `packages/harmonic-core/src/lib.rs`
- `apps/web/src/lib/wasm-loader.ts`
- `apps/web/src/hooks/useEntropy.ts`
- `apps/web/src/features/files/detectImportedPayload.ts`
- `apps/web/src/features/files/detectImportedPayload.test.ts`
- `apps/web/src/features/files/usePayloadImport.ts`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/codec/model.test.ts`
- `apps/web/src/features/workspace/SourcePanel.tsx`
- `apps/web/src/App.tsx`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `apps/web/e2e/fixtures/project/README.md`
- `apps/web/e2e/fixtures/project/src/main.rs`
- `apps/web/src/pkg/harmonic_core.d.ts`
- `apps/web/src/pkg/harmonic_core.js`
- `apps/web/src/pkg/harmonic_core_bg.wasm`
- `apps/web/src/pkg/harmonic_core_bg.wasm.d.ts`
- `docs/adr/0002-v2-project-archive.md`
- `README.md`
- `docs/TODO.md`

## 変更意図

ZIP/tar固有metadataやJSON/Base64 overheadを持ち込まず、Transport非依存の小さなschemaとして検証可能にするため。v1 APIは変更せず、既存MIDIの読取互換性を残した。

## 設計上の意図

Project schema、packet、compression、Dense transport、WASM、UIを別責務に保った。decoderは外部入力を信用せず、長さと順序を検証してからdomain objectを作る。Project testを別moduleへ分け、実装fileを300行以内に維持した。新しい外部依存は追加していない。

## 影響範囲

Project payloadの新規encodeはv2 Denseになる。TextとSource Fileのv2経路、およびv1 Source File/Project decode fallbackは維持する。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- canonical order、Unicode nested path、入力順非依存
- unsafe/non-portable path、重複path、空archive
- file count、1 file size、total source size上限
- schema/encoding、切断、余剰data、length改ざん
- Project Dense round-tripとpayload type違い
- Web codec routingとv2/v1 import判定
- Project自己検証の順序非依存比較とpath/extension/source差異検出
- ChromiumでProject 2 filesのfolder-to-MIDI-to-import round-trip

## 実行した確認コマンド

- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 81件成功
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
  - 5 VU、9秒、331 iterations、1,326 HTTP requests
  - checks 100%、request failure 0%、p95 9.89 ms
- `git diff --check`: 成功
- `npm audit --audit-level=high`: registry応答がなく90秒超で停止。2026-09-02の直近結果はhigh/critical 0件、moderate 1件、low 1件

## CIで確認される内容

Rust format、Clippy、unit test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium 4 scenarios、k6 threshold、npm high severity audit。GitHub上のCIは未pushのため未実行。

## 未解決の課題

- npm registry timeoutのため、依存監査の当日結果を取得できていない。
- JS bundleが約688 kBでcode splitting warningが残る。
- DOMPurify/Monaco由来の既知moderate 1件・low 1件は直近監査で残っている。
- Projectはbinary file、symlink、filesystem metadataを保持しない。
- 圧縮後payloadがv2 24-bit length上限を超えるProjectは、source合計32 MiB以下でもDense encodeできない。
- folder drag-and-dropそのものはDirectory Entry APIの自動fixtureがなく、folder picker経路をE2E対象とした。

## 次にやること

Phase 2 Musical Baselineのbyte-to-note mapping、tempo、duration、instrument assignmentを仕様化し、Transport contract testから実装する。npm registry復旧後に監査も再実行する。

## 次回最初に見るべきファイル

- `docs/logiscore_improvement_design.md`
- `docs/adr/0001-v2-protocol-boundaries.md`
- `docs/adr/0002-v2-project-archive.md`
- `packages/harmonic-core/src/transport/mod.rs`
- `packages/harmonic-core/src/protocol/scales.rs`
- `docs/TODO.md`

## 引き継ぎ事項

次回最初に `cargo test --all-targets --all-features --locked` と `npm run test:e2e` を実行する。v1 decode fallbackは削除しない。Musical transportからProject schemaへ依存を逆流させない。Denseの24-bit圧縮後length制約を変える場合はADR 0001/0002を更新する。既存の大規模UI変更は今回のProject差分と混ぜて再構成しない。
