# 作業報告書

## 作業日時

2026年09月01日 20時31分03秒 JST

## 作業対象

v2 Source File canonical schema、Dense MIDI codec、WASM/Web接続、v1 import互換。

## 作業目的

Filename、Extension、UTF-8 SourceをTransport非依存の決定論的形式で保持し、Webの単一ファイル経路をv2へ移行する。

## 変更内容

- 9-byte metadata headerと可変長データからなるSource File schemaを実装した。
- Filename、Extension（`.rs`形式と`Dockerfile`形式）、UTF-8、宣言長、8 MiB上限を検証する。
- v2共通packet encode/decode処理をTextとSource Fileで共有した。
- decompression bomb対策としてv2展開量に上限を設けた。
- Source FileのRust、WASM、TypeScript APIを追加した。
- WebのSource File encode/playback verificationをv2へ切り替えた。
- MIDI importはProject v1 → Text v2 → Source File v2 → Source File v1の互換順序とした。
- `lib.rs`からv2 WASM境界とv1密度計算を分離し、289行へ縮小した。

## 変更したファイル

- `packages/harmonic-core/src/payload.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/compressor.rs`
- `packages/harmonic-core/src/wasm_v2.rs`
- `packages/harmonic-core/src/density.rs`
- `packages/harmonic-core/src/lib.rs`
- `apps/web/src/pkg/*`
- `apps/web/src/lib/wasm-loader.ts`
- `apps/web/src/hooks/useEntropy.ts`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/codec/model.test.ts`
- `apps/web/src/features/files/detectImportedPayload.ts`
- `apps/web/src/features/files/detectImportedPayload.test.ts`
- `apps/web/src/features/files/usePayloadImport.ts`
- `apps/web/src/App.tsx`
- `docs/adr/0001-v2-protocol-boundaries.md`
- `docs/TODO.md`
- `README.md`

## 変更意図

Source Fileの意味をMIDI表現から分離し、将来のMusical/Acoustic Transportでも同一のcanonical bytesを再利用する。v1 MIDIのdecodeは削除せず段階移行を維持する。

## 設計上の意図

整数はbig-endian、Encoding IDはUTF-8を0として予約領域を残した。FilenameとExtensionは独立metadataとし、パス文字を拒否する。UIはWASM詳細をloaderとhookの内側へ隔離した。新規依存は追加していない。

## 影響範囲

単一Source Fileのencode、playback verification、MIDI import、protocol表示。Text v2とProject v1の経路は維持する。DB、HTTP API、認証、永続化の変更はない。

## 追加・更新したテスト

- Source File UTF-8 round-trip
- 不正Filename/Extension拒否
- schema/encoding version、truncated、trailing data拒否
- Source File Dense MIDI round-trip、決定性、Payload Type不一致拒否
- 上限付きdecompression
- v1 Dense密度境界
- Web Source File v2 routing、protocol label、v2/v1 import fallback
- Rust unit test 71件、Frontend unit test 16件
- 生成WASMでUnicode Source File round-trip

## 実行した確認コマンド

- `wasm-pack build packages/harmonic-core --target web --out-dir ../../apps/web/src/pkg`: 成功
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 71件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 16件成功
- `npm run build`: 成功
- `npm audit --audit-level=high`: 成功（high/critical 0件）
- Nodeで生成WASMのSource File encode/decode: 成功（636 bytes）
- `git diff --check`: 成功

## CIで確認される内容

Rustのformat、Clippy、test、check、build。Webのtypecheck、Prettier、ESLint、Vitest、production build、high以上のnpm audit。GitHub上は未pushのため未実行。

## 未解決の課題

- 接続可能なブラウザがないため視覚確認・クリックE2Eは未実施。
- Projectはv1経路であり、v2 archive schemaは未決定。
- `protocol/midi_gen.rs`、`protocol/mod.rs`、`useAudioEngine.ts`が300行を超える。
- Web bundleが約686KBでcode splitting警告が残る。
- DOMPurify/Monaco由来のmoderate 1件・low 1件が残る。

## 次にやること

Project canonical archive schemaを決定し、path traversal、symlink、ファイル数、個別・合計展開サイズの制約をテストで固定する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/payload.rs`
- `docs/adr/0001-v2-protocol-boundaries.md`
- `apps/web/src/features/files/projectFiles.ts`
- `docs/TODO.md`

## 引き継ぎ事項

次回最初に`cargo test --all-targets --all-features --locked`と`npm test`を実行する。Source File schemaの既存byte layoutを変更する場合はschema versionを上げる。v1 Source File decoderは既存MIDI互換のため削除しない。Project archiveでは入力pathを信用せず、復元前に全entryを検証する。
