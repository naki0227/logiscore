# 作業報告書

## 作業日時

2026年09月01日 20時24分22秒 JST

## 作業対象

Webアプリケーションの責務分離、MIDI import判定の集約、`App.tsx`の縮小。

## 作業目的

v2 UIを安全に拡張できるよう、960行のroot componentから表示・File API・decoder判定を分離し、300行以内にする。

## 変更内容

- `App.tsx`を960行から257行へ縮小した。
- Header、Source、Controls、Output、FileList、Iconsを個別componentへ分離した。
- File/MIDI importとdrag-and-dropを`usePayloadImport`へ分離した。
- v1/v2 MIDI判定を`useEntropy.decodeImportedMidi`へ集約した。
- decoder probe中の一時的なv2エラーが正常なv1 import画面へ残らないようにした。

## 変更したファイル

- `apps/web/src/App.tsx`
- `apps/web/src/components/Icons.tsx`
- `apps/web/src/features/workspace/AppHeader.tsx`
- `apps/web/src/features/workspace/SourcePanel.tsx`
- `apps/web/src/features/workspace/ControlsPanel.tsx`
- `apps/web/src/features/workspace/OutputPanel.tsx`
- `apps/web/src/features/workspace/FileListItem.tsx`
- `apps/web/src/features/files/usePayloadImport.ts`
- `apps/web/src/features/files/detectImportedPayload.ts`
- `apps/web/src/features/files/detectImportedPayload.test.ts`
- `apps/web/src/hooks/useEntropy.ts`
- `docs/TODO.md`

## 変更意図

root componentは画面状態とusecase orchestrationだけを担当し、DOM表示、ブラウザFile API、codec判定を別責務へ移した。既存UI・v1互換経路・v2 Text経路の振る舞いは維持した。

## 設計上の意図

表示componentはpropsのみを受け取る。外部入力のdecoder順序はhook内に集約し、Project v1 → Text v2 → Source File v1の順序を画面から隠蔽した。全新規ファイルは300行以内である。

## 影響範囲

Webのencode、playback verification、MIDI download/import、ZIP download、folder drop、Payload/Mode UI。Rust protocol、DB、API、認証への変更はない。

## 追加・更新したテスト

MIDI decoderの優先順位、空Projectからのfallback、v1 Source File fallback、不正MIDI拒否を4件追加した。Frontend unit test 15件とRust unit test 62件をすべて実行した。

## 実行した確認コマンド

- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 62件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 15件成功
- `npm run build`: 成功
- `npm audit --audit-level=high`: 成功（high/critical 0件、moderate 1件・low 1件は継続監視）
- `git diff --check`: 成功

## CIで確認される内容

Rustのformat、Clippy、test、check、build。Webのtypecheck、Prettier、ESLint、Vitest、production build、high以上のnpm audit。GitHub上は未pushのため未実行。

## 未解決の課題

- 接続可能なブラウザがないため視覚確認・クリックE2Eは未実施。
- `useAudioEngine.ts`が323行。
- Rustのv1互換ファイル3つが300行を超える。
- Web bundleが約686KBでcode splitting警告が残る。
- DOMPurify由来のmoderate/low脆弱性が残る。

## 次にやること

Source File canonical binary schemaを決定してADRへ追記し、v2 Source File round-tripを実装する。その前後でRust v1互換境界の大きなファイルを責務分割する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/payload.rs`
- `packages/harmonic-core/src/protocol/v2.rs`
- `docs/adr/0001-v2-protocol-boundaries.md`
- `apps/web/src/App.tsx`

## 引き継ぎ事項

次回最初に`npm test`と`cargo test --all-targets --all-features --locked`を実行する。`App.tsx`へpanel JSXやFile API処理を戻さない。MIDI decoderの互換順序は`useEntropy.decodeImportedMidi`でのみ変更する。Music/Reliableは未実装のため有効化しない。
