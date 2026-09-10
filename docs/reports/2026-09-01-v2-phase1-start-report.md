# 作業報告書

## 作業日時

2026年09月01日 19時55分23秒 JST

## 作業対象

Logiscore v2 Phase 1の開始、CI基盤、Text Payload、Binary Header、Transport境界、Dense MIDI互換adapter。

## 作業目的

改善設計書に従い、v1の完全可逆性を維持しながら、Musical/Acoustic transportを追加できるv2基盤を作る。

## 変更内容

- GitHub ActionsにRust/Webの品質ゲートを追加した。
- Rust/Webの既存format・lint負債を解消した。
- Payload TypeとText Payloadを追加した。
- 7-byte v2 Binary Headerと厳格なdecode検証を追加した。
- `Transport` traitと既存Dense MIDI adapterを追加した。
- Textを圧縮し、v2 packetとしてDense MIDI経由で完全復元するAPIを追加した。
- npm high severity脆弱性を互換更新で解消した。
- 不正なHTML head構造とTypeScriptの `any` を修正した。

## 変更したファイル

- `.github/workflows/ci.yml`
- `apps/web/package.json`, `apps/web/package-lock.json`, `apps/web/.prettierignore`
- `apps/web/index.html`, `apps/web/eslint.config.js`, `apps/web/src/App.tsx`
- `apps/web` 配下のPrettier適用対象
- `packages/harmonic-core/Cargo.toml`
- `packages/harmonic-core/src/error.rs`, `lib.rs`, `payload.rs`, `v2.rs`
- `packages/harmonic-core/src/protocol/mod.rs`, `protocol/v2.rs`
- `packages/harmonic-core/src/transport/mod.rs`, `transport/dense_midi.rs`
- `packages/harmonic-core` 配下のrustfmt適用対象
- `docs/adr/0001-v2-protocol-boundaries.md`, `docs/TODO.md`

## 変更意図

v1を直接変更すると既存MIDIとAPIを壊すため、v2を並行追加した。CIを先に整備し、以後の段階的変更を常に回帰検証できるようにした。

## 設計上の意図

Payload、Binary Header、圧縮、Transportを分離した。v2のpacketはTransport非依存であり、Dense MIDIをMusical MIDIやAcousticへ差し替えてもTextのcanonicalizationと復元ロジックを再利用できる。外部依存は追加していない。

## 影響範囲

v1公開APIと既存MIDI形式の挙動は変更していない。v2 APIはRust側のみで、Web UI/WASM公開は次回作業。CI導入により今後のpush/PRで品質ゲートが実行される。

## 追加・更新したテスト

- Payload Typeの正常値・予約値
- UTF-8 Text Payload
- Binary Headerの0/1/24-bit最大長
- 不正magic、version、短いheader、範囲外profile/length
- Dense transportの全256 byte round-trip
- 空Text、Unicode、決定論的MIDI、packet余剰データ拒否
- 未実装codec/FEC/flags profileの拒否
- 既存v1回帰を含むRust unit test合計62件

## 実行した確認コマンド

- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 62件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run format:check`: 成功
- `npm run lint`: 成功
- `npm test`: 成功（現時点ではtest fileなし）
- `npm run build`: 成功（bundle size warningあり）
- `npm audit --audit-level=high`: 成功（moderate 1件、low 1件は残存）
- `git diff --check`: 成功

## CIで確認される内容

Rustはformat、Clippy、unit test、check、build。Webはtypecheck、Prettier、ESLint、Vitest、production build、high以上のnpm auditを確認する。GitHub上の実行は未pushのため未確認。

## 未解決の課題

- v2 TextはまだWASM/Web UIから利用できない。
- Source File/Projectのcanonical binary schemaが未決定。
- Frontendには実質的なunit testがまだない。
- Monaco依存由来のmoderate/low脆弱性が各1件残る。
- 既存の巨大ファイルとWeb bundle size警告が残る。

## 次にやること

v2 Text APIをWASMへ公開し、Payload/Mode選択UIのText + Dense経路に接続する。UI変更に先立ち `App.tsx` の責務を小さく分割し、component/hook testを追加する。

## 次回最初に見るべきファイル

- `docs/logiscore_improvement_design.md`
- `docs/adr/0001-v2-protocol-boundaries.md`
- `docs/TODO.md`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/protocol/v2.rs`
- `apps/web/src/App.tsx`

## 引き継ぎ事項

次回最初に `cargo test --all-targets --all-features --locked` と `npm run typecheck && npm run lint && npm test` を実行する。v1の `encode` / `decode` と既存メタヘッダーは移行完了まで変更しない。Binary Headerのmagic/bit layout変更は後方互換性に関わるためADR更新なしに行わない。debug binaryは `autobins = false` によりCI対象外である。
