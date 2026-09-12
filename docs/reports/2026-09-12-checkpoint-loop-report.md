# 作業報告書

## 作業日時

2026年09月12日 10時55分13秒

## 作業対象

v2 field validation 3の任意位置・ループ録音checkpoint protocol。

## 作業目的

録音開始位置を先頭に合わせなくても、2周分の音から完全なcheckpointを集め、元のv2 Text packetを順序どおりに再構成できる基盤を作る。

## 変更内容

- versioned checkpoint envelopeとADR 0013を追加した。
- v2 packetを最大768-byteのchunkへ分割し、各chunkにCRC-32を付与した。
- Transfer ID、index、count、全長を検証し、順不同・重複観測を再構成する。
- 同一chunkの複数観測は完全一致数でhard voteし、tie、欠落、別transfer混在を拒否する。
- FixedFallback録音全体から同期候補を探索し、途中で切れたframeを飛ばして後続checkpointを読む。
- 2周PCMを任意位置へ回転して元Textを復元するWASM APIとPlaywright scenarioを追加した。
- 既存Reliable Project WAV E2Eのtimeoutが別scenarioへ誤適用されていた問題を修正した。

## 変更したファイル

- `docs/adr/0013-checkpoint-loop-protocol.md`
- `docs/TODO.md`
- `packages/harmonic-core/src/error.rs`
- `packages/harmonic-core/src/error_correction/crc32.rs`
- `packages/harmonic-core/src/error_correction/mod.rs`
- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/audio/fixed.rs`
- `packages/harmonic-core/src/audio/fixed_scan.rs`
- `packages/harmonic-core/src/v2.rs`
- `packages/harmonic-core/src/v2/checkpoint.rs`
- `packages/harmonic-core/src/v2/checkpoint/frame.rs`
- `packages/harmonic-core/src/v2/checkpoint/assembly.rs`
- `packages/harmonic-core/src/v2/checkpoint/audio.rs`
- `packages/harmonic-core/src/wasm_audio.rs`
- `apps/web/src/pkg/harmonic_core*`
- `apps/web/src/lib/acoustic-codec.ts`
- `apps/web/e2e/checkpoint-loop.spec.ts`
- `apps/web/e2e/codec-roundtrip.spec.ts`

## 変更意図

既存v2 packetの圧縮・FEC・Secure責務を変更せず、外側のtransport envelopeだけで任意開始、並べ替え、重複統合を実現するため。

## 設計上の意図

frame validation、assembly、audio scanを別moduleに分離した。単一frame codecと複数frame scannerも分け、全新規実装fileを300行以内に保った。Rust coreを正本にし、WASMは薄い変換、Playwrightは実ブラウザシナリオだけを担当する。新規dependencyは追加していない。

## 影響範囲

新しいcheckpoint APIとFixedFallback scanner。既存v2 packet、通常のWAV/MIDI/Secure decode、DB、外部APIにwire変更はない。

## 追加・更新したテスト

- 空payload、1/512/513-byte境界のchunk wire round-trip。
- chunk size 0/769、CRC改変、truncationの拒否。
- `Chunk 1 / Chunk 2 / Chunk 0`の回転順序と重複観測再構成。
- majority、tie、欠落、別transfer混在、全packet CRC不一致。
- 不完全frameを飛ばして次の同期frameを読むscanner。
- 途中開始の2周Text PCM round-tripと不十分な録音の拒否。
- Playwrightによる同シナリオのChromium実行。

## 実行した確認コマンド

- Rust format / clippy / 190 tests / check / build: 成功。
- Web typecheck / E2E typecheck / format / lint / Vitest 44件 / build: 成功。
- Playwright Chromium全27件: 最終再実行で成功。
- 新規dependencyなし。Cargo/npm auditは同一作業内で既知脆弱性なしを確認済み。
- k6: 642/642 checks、HTTP failure 0%、p95 13.01 ms。

## CIで確認される内容

Rust format、clippy、unit/integration test、check、build、cargo audit。Web typecheck、E2E typecheck、format、lint、Vitest、build、Playwright、k6、npm audit。両品質job成功後のVercel deploy gate。

## 未解決の課題

- checkpointは現行の長い同期framingを再利用しており、Mini Sync化は未実装。
- hard voteはCRCを通ったchunk単位であり、破損symbolのconfidenceを統合するsoft combineは未実装。
- checkpoint loopのproduction UI、WAV download、実スマホloop録音は未実装。
- FixedFallbackの音楽性と約1.20 bpsの低bitrateは改善対象。

## 次にやること

Mini Syncの長さと誤検出率をbenchmarkし、symbol confidenceを持つ観測型を設計する。その後、loop WAVのUI生成・importをPlaywrightと実スマホ録音へ接続する。

## 次回最初に見るべきファイル

`docs/adr/0013-checkpoint-loop-protocol.md`、`packages/harmonic-core/src/v2/checkpoint/audio.rs`、`packages/harmonic-core/src/audio/fixed_scan.rs`、`apps/web/e2e/checkpoint-loop.spec.ts`。

## 引き継ぎ事項

既存v2 packet内部へchunk fieldを追加しない。CRCは認証ではなく偶発誤り検出であり、Secure payloadのAEADを代替しない。Mini Sync導入時も現行checkpoint envelope version 1を維持し、音響framingだけを比較する。
