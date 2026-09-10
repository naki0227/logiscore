# 作業報告書

## 作業日時

2026年09月09日 20時34分45秒 JST

## 作業対象

v2 Phase 8 Secure Modeの設計・暗号domain着手。

## 作業目的

Password以外の共有情報を必要とせず、誤Password・改ざん時に平文を返さない暗号境界を作る。

## 変更内容

- ADR 0010でArgon2id、ChaCha20-Poly1305、CSPRNG、wire envelope、failure policyを決定した。
- `LSSE | version | KDF ID | AEAD ID | salt | nonce | ciphertext+tag` envelopeを実装した。
- 64 MiB / 3 iterations / 1 laneの固定Argon2id profileとheader AAD認証を実装した。
- password長、envelope長、version、algorithm IDを入力検証した。
- derived keyを`Zeroizing`で消去し、暗号内部errorを安全な型へ変換した。

## 変更したファイル

- `packages/harmonic-core/Cargo.toml`, `Cargo.lock`
- `packages/harmonic-core/src/secure/`
- `packages/harmonic-core/src/error.rs`, `src/lib.rs`
- `docs/adr/0010-secure-envelope.md`, `docs/TODO.md`

## 変更意図

暗号処理をUIに分散させずRust domainへ集約し、native/WASMで同じ認証保証とwire formatを使うため。

## 設計上の意図

envelopeに任意KDF costを持たせず、versionから固定設定を導出してresource exhaustionを防ぐ。headerをAADに含め、algorithm metadata改ざんも認証対象にする。production fileは148行。

## 影響範囲

現時点は新規secure domainと依存のみ。既存codec経路へは未接続。DB・外部API・ログ変更なし。

## 追加・更新したテスト

正しいpasswordのUnicode round-trip、誤password、ciphertext改ざん、空password、不正header、random salt/nonceによる非決定出力。

## 実行した確認コマンド

- `cargo search` / `cargo info`で公式crate metadataとfeatureを確認
- `cargo check --locked` 成功
- `cargo fmt --all` 成功
- `cargo test secure --all-features --locked` 3件成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings` 成功
- `git diff --check` 成功

## CIで確認される内容

Rust fmt/clippy/test/check/build。WASM/Web/Playwrightはpacket/UI統合後に全実行する。

## 未解決の課題

- secure envelopeをcompression/FEC/adaptive audio順序へ統合する。
- WASM API、Password/confirmation UX、安全なauthentication error、Playwrightを追加する。
- 新規依存を含むauditを再実行する。

## 次にやること

v2 packetにsecure flagを追加し、compress → encrypt → FEC → adaptive audioと逆順decodeを実装する。

## 次回最初に見るべきファイル

`packages/harmonic-core/src/secure/envelope.rs`、`v2_packet.rs`、`v2/adaptive_audio.rs`、`v2/adaptive_decode.rs`、`docs/adr/0010-secure-envelope.md`。

## 引き継ぎ事項

acoustic profile IDはflags下位3 bitを使用中のため、Secure flagはbit 7を使う。認証失敗ではpassword誤り・tag改ざんを区別しない。browser確認はPlaywrightスクリプトとして自動実行する。Secure依存は新規のためphase完了時にlicense/advisory/auditを確認する。
