# 作業報告書

## 作業日時

2026年09月10日 22時05分58秒

## 作業対象

v2 Phase 9の実録音6指標telemetryとスマートフォン収録用transmission fixture。

## 作業目的

MacBook Speaker→1m→iPhone Voice Memosの実M4Aを受け取った直後に、手作業なしで復元可否と6共通指標を測定できる状態にする。

## 変更内容

- Rust decoder境界にText実録音のprofile探索とraw packet比較を追加した。
- Final Recovery Rate、Raw Symbol Accuracy、Corrected ErrorsをWASMへ公開した。
- Web Audioで読み込んだM4A/WAVから6共通指標を算出するWeb adapterを追加した。
- Playwrightが計測結果をJSON artifactとして保存するシナリオを追加した。
- 収録時に再生する固定Text WAVを生成し、元Textへの完全復元を自動検証した。
- corpusのfixture名、日時、corrected error countのruntime validationを強化した。
- 実録音原本のSHA-256をmanifestで固定し、Playwright実行前に改変を検出する。
- 提供されたiPhone Voice Memos原本をcorpusへ登録し、FixedFallbackの弱いsymbol検出floorを実測に合わせた。
- Playwrightで実空気伝送のText完全復元と6指標を確認し、entryをverifiedへ進めた。

## 変更したファイル

- `packages/harmonic-core/src/benchmark/recording.rs`
- `packages/harmonic-core/src/benchmark.rs`
- `packages/harmonic-core/src/benchmark/channel.rs`
- `packages/harmonic-core/src/v2/adaptive_audio.rs`
- `packages/harmonic-core/src/wasm_adaptive.rs`
- `packages/harmonic-core/examples/generate_recording_fixture.rs`
- `apps/web/src/pkg/harmonic_core*`
- `apps/web/src/lib/acoustic-codec.ts`
- `apps/web/src/features/audio/recordingMetrics.ts`
- `apps/web/src/features/audio/recordingMetricsModel.ts`
- `apps/web/src/features/audio/recordingMetricsModel.test.ts`
- `apps/web/e2e/measure-recording.ts`
- `apps/web/e2e/recording-formats.spec.ts`
- `apps/web/e2e/recording-corpus.ts`
- `apps/web/e2e/recording-corpus.spec.ts`
- `apps/web/e2e/fixtures/recordings/corpus.json`
- `apps/web/e2e/fixtures/recordings/capture-text-fixed-fallback.wav`
- `apps/web/e2e/fixtures/recordings/iphone-record-1m-quiet-001.m4a`
- `apps/web/e2e/fixtures/recordings/README.md`
- `docs/adr/0012-benchmark-metrics.md`
- `docs/TODO.md`

## 変更意図

実録音の成功判定と性能値を同じdecoder結果から生成し、「ブラウザで戻った」と「benchmarkで測れた」を分離しないため。固定再生WAVにより物理収録条件の入力も再現可能にした。

## 設計上の意図

packet比較はRust benchmark層、ファイルdecodeと時間計測はWeb adapter、シナリオとartifact生成はPlaywrightへ分離した。production UIへbenchmark専用状態を持ち込まず、codec wire formatも変更していない。fixture名をbasename相当へ制限し、manifestからcorpus外ファイルを読めないようにした。

## 影響範囲

Rust benchmark/WASM API、Webのbenchmark helper、Playwright E2E、収録corpus。既存protocol、DB、外部API、production UIには変更なし。

## 追加・更新したテスト

- clean recordingの100% raw/final recovery。
- silenceでprofile未検出、0% recovery。
- sample rate 0と非有限sampleの拒否。
- WASM telemetry JSONの正常・異常schema。
- Opus fixtureの6指標測定。
- 収録用WAVの元Text完全復元と6指標測定。
- corpus外pathと非対応拡張子の拒否。
- 強いframing後にpayload symbolだけが弱くなった信号の復元。
- 実iPhone M4AのPlaywright完全復元とverified measurement比較。
- fixture path traversalと非対応拡張子のruntime拒否。

## 実行した確認コマンド

- `cargo fmt --all -- --check`: 成功。
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功。
- `cargo test --all-targets --all-features --locked`: 180件成功。
- `cargo check --all-targets --all-features --locked`: 成功。
- `cargo build --all-targets --all-features --locked`: 成功。
- Web typecheck / E2E typecheck / format / lint / Vitest 44件 / build: 成功。
- `npm run test:e2e`: Chromium 25件成功。実M4Aの完全復元を含む。
- `cargo audit`: 126依存を検査し既知脆弱性なし。
- `npm audit --audit-level=high`: 既知脆弱性0件。
- k6: 650/650 checks、HTTP failure 0%、p95 13.23 ms。

## CIで確認される内容

Rust format、clippy、unit/integration test、check、build、cargo audit。Web typecheck、E2E typecheck、format、lint、Vitest、build、Playwright、k6、npm audit。両品質job成功後のVercel deploy gate。

## 未解決の課題

- Decode Timeは実行環境依存のためverified corpusの厳密回帰値には使わない。
- FixedFallbackは約136.72秒を要するため、次段階ではbitrateと耐性のtrade-offを測る必要がある。

## 次にやること

任意位置・ループ録音のchunk/checkpoint protocolを設計する。並行してTextの距離・端末条件とSource File / Projectへcorpusを拡張する。

## 次回最初に見るべきファイル

`apps/web/e2e/fixtures/recordings/README.md`、`apps/web/e2e/fixtures/recordings/corpus.json`、`apps/web/e2e/recording-corpus.spec.ts`。

## 引き継ぎ事項

実録音をtrim、normalize、transcodeしない。失敗録音も消さない。手動ブラウザ確認は完了判定に使わない。今回のverified値はFinal 100%、Raw 99.86462093862816%、Corrected 3 bit、1.2024851359476252 bps、139710.66666666666 ms。端末型番は原本metadataから取得できなかったため推測せず未特定としている。
