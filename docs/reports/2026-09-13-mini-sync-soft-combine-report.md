# 作業報告書

## 作業日時

2026年09月13日 01時48分41秒

## 作業対象

v2 field validation 3のcheckpoint Mini Syncと複数loop soft combine。

## 作業目的

checkpointごとの同期overheadを短縮し、各loopが別symbolで壊れて単独CRCに失敗しても、複数観測から安全に復元できるようにする。

## 変更内容

- checkpoint専用のMini Fixed PCM codecを追加した。
- 同期toneと長さheaderを短縮し、10-bitで最大1023-byte frameを表現した。
- 旧full-sync checkpoint録音はdecoder fallbackで後方互換を維持した。
- 各nibbleの最大・次点energy差を0〜1のconfidenceとして保持した。
- 同一transfer/chunkの観測をconfidence付きnibble voteで統合し、chunk CRCと全packet CRCを再検証した。
- 3周それぞれ別nibbleを音響的に置換し、全単独loopがCRC不合格でも統合復元するPlaywright scenarioを追加した。

## 変更したファイル

- `packages/harmonic-core/src/audio/mini.rs`
- `packages/harmonic-core/src/audio/mini/tests.rs`
- `packages/harmonic-core/src/audio/fixed_scan.rs`
- `packages/harmonic-core/src/audio/mod.rs`
- `packages/harmonic-core/src/v2/checkpoint.rs`
- `packages/harmonic-core/src/v2/checkpoint/audio.rs`
- `packages/harmonic-core/src/v2/checkpoint/frame.rs`
- `packages/harmonic-core/src/v2/checkpoint/soft.rs`
- `apps/web/src/pkg/harmonic_core.d.ts`
- `apps/web/src/pkg/harmonic_core_bg.wasm`
- `apps/web/src/pkg/harmonic_core_bg.wasm.d.ts`
- `apps/web/e2e/checkpoint-loop.spec.ts`
- `docs/adr/0013-checkpoint-loop-protocol.md`
- `docs/logiscore_improvement_design.md`
- `docs/TODO.md`

## 変更意図

任意開始録音ではcheckpointを細かく置く必要があるため、旧3.6秒相当のfull sync framingを繰り返す負担を減らす。壊れた観測もCRC前に持つsymbol情報を捨てず、複数周を復元率向上へ利用する。

## 設計上の意図

既存v2 packetとcheckpoint envelope v1は変更せず、交換可能な音響framingだけを追加した。soft combineはmetadataが妥当な同一chunkに限定し、CRC検証を迂回しない。新規dependencyはなく、実装・テストfileはすべて300行以内に分割した。

## 影響範囲

checkpoint loop PCMのencodeはMini Syncへ切り替わる。decodeはMini Syncを優先し、観測がない場合だけ旧Fixed PCMへfallbackする。通常のReliable WAV、MIDI、Secure packet、DB、外部APIには変更がない。

## 追加・更新したテスト

- Mini Syncの複数frame scan、任意途中開始、1023-byte境界。
- 空frameでMini Syncが旧full syncの1/4未満であること。
- 旧full-sync checkpoint音声の互換decode。
- confidence shape不正と同点voteの拒否。
- 3つのCRC不合格byte観測からのdomain soft combine。
- 3周すべて別symbol破損からのPCM end-to-end復元。
- Playwright Chromiumで同じ破損loop復元と全単独loop拒否。

## 実行した確認コマンド

- `cargo fmt --all -- --check`: 成功。
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功。
- `cargo test --all-targets --all-features --locked`: 197件成功。
- `cargo check --all-targets --all-features --locked`: 成功。
- `cargo build --all-targets --all-features --locked`: 成功。
- `cargo audit`: 126 dependenciesを検査しadvisoryなし。
- `wasm-pack build . --target web --out-dir ../../apps/web/src/pkg`: 成功。
- `npm run typecheck` / `npm run typecheck:e2e`: 成功。
- `npm run format:check` / `npm run lint`: 成功。
- `npm test`: Vitest 44件成功。
- `npm run build`: 成功。JS bundle 706.82 kBの既知warningあり。
- `npm audit --audit-level=high`: 0 vulnerabilities。
- `npm run test:e2e`: Playwright Chromium全28件成功。
- k6 static delivery: 642/642 checks、HTTP failure 0%、p95 12.64 ms。

## CIで確認される内容

Rust format、clippy、全test、check、build、cargo audit。Web typecheck、E2E typecheck、format、lint、Vitest、build、Playwright全件、k6、npm audit。Rust/Web成功後のVercel production gate。

## 未解決の課題

- checkpoint loopのproduction UIとWAV download/importは未実装。
- 任意開始・複数周の実iPhone録音は未収録。
- Source File、Project、Secure payloadのcheckpoint WASM境界は未実装。
- FixedFallbackの単音シンセ感、不気味さ、低bitrateはMini Syncとは別に音楽profileで改善が必要。
- metadata自体が壊れたsoft observationは安全にgroup化できないため破棄する。

## 次にやること

Text checkpoint loopをproduction UIとWAV download/importへ接続し、Playwrightで操作を自動化する。その後、任意位置から2周以上をiPhone Voice Memosで実収録してcorpusへ追加する。

## 次回最初に見るべきファイル

`docs/adr/0013-checkpoint-loop-protocol.md`、`packages/harmonic-core/src/v2/checkpoint/audio.rs`、`apps/web/src/hooks/useAcousticCodec.ts`、`apps/web/e2e/checkpoint-loop.spec.ts`。

## 引き継ぎ事項

checkpoint envelope v1と既存v2 packetは変更しない。soft combine後もCRCを必ず通す。FixedFallbackの音色改善を同期framing変更へ混ぜず、音楽性・bitrate・耐性を同じbenchmark frameworkで比較する。ブラウザ動作確認はPlaywright scriptで自動実行する。
