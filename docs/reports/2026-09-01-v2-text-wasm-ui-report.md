# 作業報告書

## 作業日時

2026年09月01日 20時12分15秒 JST

## 作業対象

v2 Text PayloadのWASM公開、Web UI接続、codec routing、フォルダ走査分離。

## 作業目的

Phase 1のText経路をRust内部だけでなくブラウザから利用可能にし、v1 File/Projectとの段階移行UIを提供する。

## 変更内容

- `encode_text_v2_wasm`、`decode_text_v2_wasm`、`get_v2_version` を公開した。
- wasm-packでJS、型定義、WASM binaryを再生成した。
- Web loaderと`useEntropy`へv2 Text encode/decodeを追加した。
- Payload（Text / Source File / Project）とMode（Auto / Fast）selectorを追加した。
- Textはv2 Dense、File/Projectはv1 Denseへ明示的にrouteする純粋関数を追加した。
- 未実装のMusic/ Reliableはdisabled表示とし、ロジック側でも拒否する。
- MIDI importはProject v1 → Text v2 → Source File v1の順で安全に判定する。
- フォルダ走査、拡張子allowlist、ignore処理を`features/files`へ分離した。
- 文字化けしていたエラーアイコンを修正した。
- Vite configを`import.meta.dirname`へ更新し、native loader互換警告を解消した。

## 変更したファイル

- `packages/harmonic-core/src/lib.rs`
- `apps/web/src/pkg/harmonic_core.js`
- `apps/web/src/pkg/harmonic_core.d.ts`
- `apps/web/src/pkg/harmonic_core_bg.wasm`
- `apps/web/src/pkg/harmonic_core_bg.wasm.d.ts`
- `apps/web/src/lib/wasm-loader.ts`
- `apps/web/src/hooks/useEntropy.ts`
- `apps/web/src/features/codec/CodecSelector.tsx`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/codec/model.test.ts`
- `apps/web/src/features/files/projectFiles.ts`
- `apps/web/src/features/files/projectFiles.test.ts`
- `apps/web/src/App.tsx`, `apps/web/src/App.css`
- `docs/TODO.md`

## 変更意図

Textだけをv2へ移行し、既存File/Projectをv1のまま維持することで、ユーザーが既存機能を失わずに新プロトコルを試せるようにした。

## 設計上の意図

UIがWASM関数名へ直接依存しないようloaderとhookを境界にした。PayloadとModeからcodec routeを決める処理はReactから分離し、Phase 2以降のroute追加をunit testで保護する。外部入力はpayloadごとのdecoderを順番に試し、いずれも検証できなければ内部詳細を表示せず拒否する。

## 影響範囲

Text選択時のencode/playback verification/MIDI importはv2経路になる。Source FileとProjectはv1互換経路を維持する。DB、外部API、認証、永続化の変更はない。

## 追加・更新したテスト

- PayloadごとのAuto route 3件
- Text Fast route
- 未実装Music/Reliable拒否
- v1/v2 protocol label
- 拡張子抽出、extensionless filename、allowlist拒否
- Frontend unit test合計11件
- 既存Rust unit test 62件
- 生成WASMを直接初期化したUnicode Text round-trip

## 実行した確認コマンド

- `wasm-pack build --target web --release --out-dir ../../apps/web/src/pkg`: 成功
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 62件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run format:check`: 成功
- `npm run lint`: 成功
- `npm test`: 11件成功
- `npm run build`: 成功
- Nodeで生成WASMのText encode/decode: 成功（663 bytes）
- Vite local server起動: 成功

## CIで確認される内容

Rustのformat、Clippy、test、check、buildと、Webのtypecheck、Prettier、ESLint、Vitest、production build、high以上のnpm audit。GitHub上は未pushのため未実行。

## 未解決の課題

- 接続可能なブラウザがなかったため、実画面の視覚確認とクリックE2Eは未実施。
- `App.tsx` は960行あり、300行制約を未達。フォルダ走査のみ分離済み。
- Source File/Project canonical binary schemaは未決定。
- 約685KBのbundle size警告が残る。
- Monaco/DOMPurify由来のmoderate 1件・low 1件が残る。

## 次にやること

MIDI importと3つのpanelを`App.tsx`から分離して300行以内にする。その後Source File canonical formatをADRへ追記し、Phase 1を完了させる。

## 次回最初に見るべきファイル

- `apps/web/src/App.tsx`
- `apps/web/src/features/codec/model.ts`
- `apps/web/src/features/files/projectFiles.ts`
- `packages/harmonic-core/src/payload.rs`
- `docs/adr/0001-v2-protocol-boundaries.md`

## 引き継ぎ事項

次回最初に`npm test`と`cargo test --all-targets --all-features --locked`を実行する。Textは必ずv2 Binary Header、File/Projectは移行完了までv1を使う。Music/Reliable optionは実装前に有効化しない。WASM再生成時はrustup stableの`wasm32-unknown-unknown` targetを使用する。
