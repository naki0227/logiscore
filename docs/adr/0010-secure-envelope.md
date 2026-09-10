# ADR 0010: Password由来鍵とAEADをversioned Secure Envelopeに格納する

- Status: Accepted
- Date: 2026-09-09

## 背景

Secure Modeでは、音声とは別に共有する情報をPasswordだけにし、誤Passwordまたは改ざん時に平文を一切返さない必要がある。native Rustとbrowser WASMで同じwire formatを扱う。

## 課題

- 人間が入力する低entropy passwordを直接鍵にしない。
- saltとnonceを暗号学的乱数で毎回生成する。
- KDF/AEAD/version/salt/nonceを自己完結したenvelopeに含める。
- attacker-controlled KDF parameterによるmemory/CPU DoSを避ける。
- password・derived key・平文をログへ出さない。

## 選択肢

1. Web Crypto PBKDF2 + AES-GCMをUIだけで実装する。
2. Argon2id + AES-GCMをRust/WASMで実装する。
3. Argon2id + ChaCha20-Poly1305をRust domainに実装し、WASMは薄い境界にする。

## 採用した案

選択肢3を採用した。

- RustCrypto `argon2 0.6`のArgon2id v19、64 MiB、3 iterations、1 lane、32-byte keyを固定KDF profile 1とする。
- RustCrypto `chacha20poly1305 0.11`のRFC 8439 ChaCha20-Poly1305（32-byte key、12-byte nonce、16-byte tag）をAEAD profile 1とする。
- `getrandom 0.4`のOS CSPRNG / `wasm_js` Web Crypto backendで16-byte saltと12-byte nonceを生成する。
- `zeroize 1`でderived key materialをdrop時に消去する。
- envelopeは`magic(4) | version(1) | kdf_id(1) | aead_id(1) | salt(16) | nonce(12) | ciphertext+tag`とし、固定header全体をAEAD AADにする。
- decoderはversion/KDF/AEADを固定値検証し、envelope由来の任意KDF parameterを実行しない。
- passwordはUTF-8 1〜1024 bytes、envelopeは既存payload上限内に制限する。
- 認証失敗は原因を区別せず単一の安全なerrorへ変換する。

## 採用理由

Argon2idはpassword KDF、ChaCha20-Poly1305はAEADとして設計書に一致する。Rust domainへ集約するとnative/WASMの暗号実装とwire formatが一つになり、Web Crypto依存の分岐を持たずにテストできる。ChaCha20はsoftware実装でも安定した性能を得やすい。

## メリット

- password以外のsalt/nonce/versionを音声内で自己完結できる。
- metadataもAADで認証される。
- native/browserで同じtest vectorとfailure policyを使える。
- KDF costをwire versionで安全に見直せる。

## デメリット

- WASM bundleと初回暗号処理時間、memory使用量が増える。
- 64 MiBを確保できない低memory端末ではSecure Modeが失敗する可能性がある。
- nonce衝突確率はCSPRNG品質に依存する。
- KDF parameter変更は新しいprofile/versionが必要になる。

## 代替案を採用しない理由

- PBKDF2は広く利用可能だが、memory-hardではなくpassword攻撃耐性でArgon2idに劣る。
- AES-GCMは有力だが、今回のpure Rust/WASMと設計書の第一候補にはChaCha20-Poly1305が適する。
- 暗号をUIだけに置くとnative APIと挙動が分裂する。

## 将来的な見直し条件

- Phase 9で対象browserのKDF時間またはmemory失敗率が許容範囲を外れる場合。
- Argon2またはChaCha20-Poly1305に重大なcryptographic advisoryが出た場合。
- hardware AESが主要target全体で明確に有利になった場合。
- multi-recipient/key-managementが必要になった場合。
