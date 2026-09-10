# 作業報告書

## 作業日時

2026年09月08日 22時10分35秒 JST

## 作業対象

v2 Phase 5 Error Correction、FEC profile `1`、Reliable PCMのRust/WASM/Playwright境界。

## 作業目的

音響経路で発生する単一bit errorとburst errorを訂正し、訂正不能または見逃した誤りをpayload展開前に拒否できる保護層を作る。

## 変更内容

- IEEE CRC-32を追加した。
- Hamming SECDED (13,8) encoder/decoderを追加した。
- 8-codeword単位のbit interleavingを追加した。
- protected stream 3-copyとbit-majority復元を追加した。
- correction countを返す`Recovery` domain結果を追加した。
- v2 packetのFEC profile `1` build/decodeを追加した。
- Text、Source File、ProjectのReliable PCM Rust/WASM/TypeScript APIを追加した。
- Reliable PCMの全payload browser round-tripをPlaywrightへ追加した。

## 変更したファイル

- `packages/harmonic-core/src/error_correction/mod.rs`
- `packages/harmonic-core/src/error_correction/crc32.rs`
- `packages/harmonic-core/src/error_correction/hamming.rs`
- `packages/harmonic-core/src/error_correction/interleave.rs`
- `packages/harmonic-core/src/error_correction/repetition.rs`
- `packages/harmonic-core/src/error_correction/tests.rs`
- `packages/harmonic-core/src/error.rs`
- `packages/harmonic-core/src/lib.rs`
- `packages/harmonic-core/src/v2_packet.rs`
- `packages/harmonic-core/src/v2/pcm.rs`
- `packages/harmonic-core/src/wasm_v2.rs`
- `apps/web/src/lib/wasm-loader.ts`
- `apps/web/src/pkg/harmonic_core*`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `README.md`
- `docs/adr/0007-error-correction-profile.md`
- `docs/TODO.md`

## 変更意図

Phase 4のsignal detectionとpayload serializationの間に独立した保護境界を置き、後続の物理音響試験でsignal errorとdata integrity failureを分けて評価するため。

## 設計上の意図

既存FEC profile `0`を変更せず、profile `1`を明示選択する。CRC、SECDED、Interleaving、Repetitionを小さい責務別moduleへ分離した。新規依存は追加せず、Rust nativeとWASMで同じ実装を使用する。全追加production fileは300行以内である。

## 影響範囲

Reliable PCM APIとFEC profile `1`が追加される。既存Dense MIDI、Musical profile 1、Rhythmic profile 2、unprotected PCM、v1 import互換性は変更しない。DB、HTTP API、認証、永続化変更はない。

## 追加・更新したテスト

- CRC-32標準test vector
- 全byte値・空data・任意dataのFEC round-trip
- 任意位置のsingle post-vote bit correction property test
- 1-copy内の連続burst recovery
- SECDED single-bit correction / double-bit rejection
- SECDEDが見逃す3-bit errorのCRC rejection
- malformed/truncated frame rejection
- v2 packet profile分離とbody error correction
- Reliable PCMで壊れたpacket bodyを復元するintegration test
- Text／Source File／ProjectのReliable PCM round-trip
- Playwright Chromiumで全Reliable PCM payloadのWASM round-trip

## 実行した確認コマンド

- `wasm-pack build . --target web --out-dir ../../apps/web/src/pkg`: 成功
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 135件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run typecheck:e2e`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 22件成功
- `npm run build`: 成功。WASM約369 kB、JS約689 kB
- `npm run test:e2e`: Chromium 9件成功
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、311 iterations、1,246 HTTP requests
  - checks 100%、request failure 0%、p95 15.2 ms
- `npm audit --audit-level=high`: 成功。high/critical 0件、moderate 2件、low 1件
- `git diff --check`: 成功

## CIで確認される内容

Rust format、Clippy、test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- Binary Header自体はFEC対象外で、破損時はpacketを拒否する。
- profile `1`は容量overheadが大きく、環境別強度は未対応。
- WAV/M4A、resampling、実speaker/microphone試験はPhase 6。
- Reliable mode UIは音声file/recording workflow完成までdisabledのまま。
- JS bundleのcode splitting warningとmoderate/low依存脆弱性が残る。

## 次にやること

Phase 6としてWAV入出力、sample-rate変換、人工channel劣化、録音file import/exportを実装する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/audio/decode.rs`
- `packages/harmonic-core/src/error_correction/mod.rs`
- `docs/adr/0007-error-correction-profile.md`
- `docs/TODO.md`

## 引き継ぎ事項

FEC profile `1`の順序、SECDED codeword、interleave depth、repetition count、CRC byte orderはwire compatibilityである。変更時は新FEC profileを使う。CRCは認証ではなく、Secure Modeでは必ずAEADを使う。ブラウザ確認は必ずPlaywrightスクリプトとして追加し自動実行する。

既存worktreeにはPhase 1〜5が未コミットで共有ファイルが重なる。コミット時は `ci/test基盤`、`Phase 1 protocol/payload`、`Phase 2 musical`、`Phase 3 rhythmic`、`Phase 4 PCM`、`Phase 5 FEC` の順にpatch stagingし、各実装と対応テスト・ADRを同じコミットへ含める。
