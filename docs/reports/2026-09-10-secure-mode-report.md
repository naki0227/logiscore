# 作業報告書

## 作業日時

2026年09月10日 09時07分55秒

## 作業対象

v2 Phase 8 Secure Mode、Rust/WASM暗号境界、Web Password UX、自動ブラウザシナリオ、依存監査。

## 作業目的

Password以外の共有情報を必要とせず、改ざんまたは誤Passwordから平文を一切返さない認証付き暗号WAVを、Text・Source File・Projectで利用可能にする。

## 変更内容

- Argon2id（64 MiB、3 iterations、1 lane）とChaCha20-Poly1305によるversioned Secure Envelopeを追加した。
- OS/Web Crypto CSPRNGでsaltとnonceを毎回生成し、headerをAADとして認証した。
- `compress -> encrypt -> FEC -> PCM/WAV`と逆経路を実装し、secure flagとadaptive profileを同じv2 headerで扱った。
- payload typeを認証後に一度だけ判定する統合Secure decoderをRust/WASMへ公開した。
- Secure Mode、Password/確認入力、UTF-8 1〜1024 byte検証、メモリ内保持の説明をWeb UIへ追加した。
- 誤Passwordと破損を同じ一般エラーへ変換し、暗号詳細や平文をUIへ出さないようにした。
- PlaywrightへSecure専用シナリオを追加し、全ブラウザ確認をスクリプトで自動実行した。
- RustSecで検出した`crossbeam-epoch`、`anyhow`、`rand`を修正版へlock更新し、CIへ`cargo audit`を追加した。

## 変更したファイル

- `packages/harmonic-core/src/secure/`
- `packages/harmonic-core/src/v2_secure_packet.rs`
- `packages/harmonic-core/src/v2/secure_audio.rs`
- `packages/harmonic-core/src/v2/secure_decode.rs`
- `packages/harmonic-core/src/wasm_secure.rs`
- `packages/harmonic-core/src/v2_packet.rs`、`src/v2_packet/tests.rs`
- `packages/harmonic-core/Cargo.toml`、`Cargo.lock`
- `apps/web/src/lib/secure-acoustic-codec.ts`
- `apps/web/src/hooks/useAcousticCodec.ts`、`useSecureMode.ts`
- `apps/web/src/features/secure/`
- `apps/web/src/features/codec/`、`features/workspace/ControlsPanel.tsx`
- `apps/web/src/App.tsx`、`App.css`
- `apps/web/e2e/secure-mode.spec.ts`
- `.github/workflows/ci.yml`
- `docs/adr/0010-secure-envelope.md`、`docs/TODO.md`

## 変更意図

暗号処理をRust domainへ集約してnative/WASMのwire formatを一致させ、UIはPassword入力と安全なエラー表示だけを担当させるため。Secure payloadの自動判定はArgon2idの重い鍵導出をpayload候補ごとに繰り返さないために追加した。

## 設計上の意図

- KDF parameterをenvelopeから受け取らず固定profileにし、攻撃者がCPU/メモリ量を増幅できないようにした。
- 暗号文をFEC対象にし、訂正後のAEAD認証で完全性を最終判定する。
- Passwordはstate以外へ保存せず、localStorage/sessionStorage/logへ出さない。
- 暗号境界、packet、音声、WASM、UIを分離し、純粋なPassword検証と外部JSON検証をunit test可能にした。

## 影響範囲

v2 WAVの新しいSecure profile、Web codec選択、WASM bundle、Cargo依存、CI依存監査。既存Dense/Musical/Reliable packetと旧Reliable WAVの互換経路は維持される。DB/API変更はない。

## 追加・更新したテスト

- Rust: envelope正常系、Unicode、誤Password、改ざん、random salt/nonce、不正header、secure packet/FEC、全payload WAV往復。
- Vitest: Secure codec routing/label、UTF-8 byte境界、確認不一致、WASM JSON responseの全shapeと不正入力。
- Playwright: Password gate/storage非保存、正誤PasswordのWAV import、非開示、randomized output、全Secure payloadのbrowser-WASM往復。

## 実行した確認コマンド

- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features -- -D warnings`: 成功
- `cargo test --all-features`: 169件成功
- `cargo check --all-targets --all-features`: 成功
- `cargo build --all-targets --all-features`: 成功
- `wasm-pack build --target web --out-dir ../../apps/web/src/pkg --no-opt`: 成功
- `npm run typecheck` / `npm run typecheck:e2e`: 成功
- `npm run format:check` / `npm run lint`: 成功
- `npm test`: 42件成功
- `npm run build`: 成功。JS 706.82 kB、WASM 648.67 kBの既知warningあり
- `npm run test:e2e`: Chromium 19件成功
- `npm audit --audit-level=moderate`: 0 vulnerabilities
- `cargo audit`: 126 dependencies、脆弱性・warning 0件
- k6 2.2.0: 5 VU、345 iterations、1,382 requests、checks 100%、failure 0%、p95 5.87 ms

## CIで確認される内容

Rust fmt、Clippy、test、check、build、RustSec audit。Web typecheck、E2E typecheck、Prettier、ESLint、Vitest、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- Phase 9でKDF時間、encode/decode時間、出力サイズ、音声条件別成功率を定量化する。
- JS 706.82 kB、WASM 648.67 kBのbundleをcode splittingする。
- 300行超のv1互換ファイルと`useAudioEngine.ts`を振る舞いを変えず分割する。

## 次にやること

Phase 9 Benchmark fixture、測定runner、結果formatを設計し、通常/Reliable/Secureと各adaptive profileを比較する。

## 次回最初に見るべきファイル

`docs/logiscore_improvement_design.md`、`docs/TODO.md`、`packages/harmonic-core/src/v2/secure_decode.rs`、`apps/web/tests/load/static-assets.js`。

## 引き継ぎ事項

次回最初に`cargo test --all-targets --all-features --locked`と`npm run test:e2e`を実行する。ブラウザ確認は必ずPlaywrightスクリプトとして実装し自動実行する。KDF profileとSecure Envelope headerは互換性境界なので、変更時はADRとversionを更新する。v1互換分割をPhase 9機能変更と同じコミットに混ぜない。
