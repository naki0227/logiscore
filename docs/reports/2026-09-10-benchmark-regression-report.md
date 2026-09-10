# 作業報告書

## 作業日時

2026年09月10日 21時36分32秒

## 作業対象

v2 Phase 9のfixture size別baselineとregression comparison。

## 作業目的

32/128/256-byteの変化を同じschemaで保存し、回帰時に機能劣化と性能変動を区別できるようにする。

## 変更内容

- 32-byteと256-byteのschema-v2 baselineを追加し、既存128-byteと合わせて3サイズにした。
- 2つのreportをrow名で比較するregression moduleを追加した。
- clean round-trip、Final Recovery Rate、Raw Symbol Accuracyの悪化をfunctional regressionとして判定する。
- Corrected Errors、Payload Bitrate、Decode Time、Playback Durationは差分を出すが、自動失敗条件にはしない。
- 比較用CLI exampleを追加し、functional regression時だけnon-zero終了するようにした。

## 変更したファイル

- `packages/harmonic-core/src/benchmark.rs`
- `packages/harmonic-core/src/benchmark/midi.rs`
- `packages/harmonic-core/src/benchmark/regression.rs`
- `packages/harmonic-core/examples/compare_v2_benchmarks.rs`
- `docs/benchmarks/2026-09-10-v2-text-32.json`
- `docs/benchmarks/2026-09-10-v2-text-256.json`
- `README.md`、`docs/TODO.md`

## 変更意図

hardware負荷で揺れる時間値を機能失敗と混同せず、復元性能の悪化だけを確実に検出するため。

## 設計上の意図

schema version、fixture size、row数を先に検証する。wall-clockは比較結果へ残すが閾値化しない。Secureのrandom salt/nonceによる音長変動もperformance差分として観測し、決定論的と偽らない。

## 影響範囲

Rust benchmark API、QA example、baseline JSON、文書。codec wire format、通常decoder、WASM、Web UI、DBに変更はない。

## 追加・更新したテスト

- Final Recovery低下をfunctional regressionとして検出する正常系。
- Decode Time増加は差分表示のみでfunctional regressionにしないケース。
- fixture size不一致の拒否。

## 実行した確認コマンド

- `cargo test benchmark:: --locked`: 6件成功。
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功。
- `cargo run --release --locked --example v2_benchmark -- 32 3`: 成功。
- `cargo run --release --locked --example v2_benchmark -- 256 3`: 成功。
- `cargo run --release --locked --example compare_v2_benchmarks -- <same baseline twice>`: functional regressionなし。
- Rust fmt / Clippy / test 175件 / check / build: 成功。
- RustSec auditは直前のtelemetry commitで126 dependencies、脆弱性なし。依存変更なし。
- Web code変更なし。全Web gateはpush後のCIで再実行する。

## CIで確認される内容

Rust fmt、Clippy、unit/all-target test、check、build、audit。Web全ゲート、Playwright、k6、npm audit、成功後のVercel gate。

## 未解決の課題

- performance regressionの自動閾値はhardware別baselineなしでは設定しない。
- Corrected Errorsは通信路の誤り量にも依存するため、単純な増減を良否判定しない。
- Secure WAVの出力音長は暗号randomnessにより実行ごとに変動する。
- 実smartphone recording corpusは未収集。

## 次にやること

MacBook Speaker、1m、iPhone Voice Memos、M4Aを最初の固定条件とするcorpus metadata schema、命名規則、収録手順、fixture検証runnerを追加する。

## 次回最初に見るべきファイル

`docs/logiscore_improvement_design.md`、`docs/TODO.md`、`packages/harmonic-core/src/benchmark.rs`。

## 引き継ぎ事項

実録音がない状態でsmartphone round-trip成功と記録しない。report比較は同じschema version・fixture sizeだけで行う。performance値は異なるhardware間のpass/failに使わない。
