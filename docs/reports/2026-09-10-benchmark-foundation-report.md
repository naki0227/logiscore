# 作業報告書

## 作業日時

2026年09月10日 09時42分16秒

## 作業対象

v2 Phase 9 Benchmark、Rust codec全経路、決定論的channel model。

## 作業目的

Music Quality設定、Bitrate、Playback Time、Noise Robustness、Distance相当条件のトレードオフを、再実行可能なJSONで定量化する。

## 変更内容

- Dense MIDI、Rhythmic MIDI、7 Acoustic Profiles、Secure WAVの共通benchmark reportを実装した。
- output bytes、encode/decode中央値、playback ms、payload bitrate、clean round-trip、music weight、polyphony、FEC、channel成功数を出力する。
- clean、軽ノイズ、会話相当echo、中ノイズ、距離相当gain/noise/clip/offset/echoの5条件を決定論的に適用する。
- 128-byte fixture、3 iterationsのrelease測定結果を保存した。

## 変更したファイル

- `packages/harmonic-core/src/benchmark.rs`
- `packages/harmonic-core/src/benchmark/channel.rs`
- `packages/harmonic-core/examples/v2_benchmark.rs`
- `packages/harmonic-core/src/lib.rs`
- `docs/benchmarks/2026-09-10-v2-text-128.json`
- `docs/adr/0012-benchmark-metrics.md`
- `docs/TODO.md`、`README.md`

## 変更意図

単なるmicro benchmarkではなく、利用者が受け取るMIDI/WAV全体と実際のdecoderを測り、冗長性・再生時間・復元性を同じreportで比較するため。

## 設計上の意図

- fixtureとchannelノイズ系列を決定論的にし、結果差を追跡可能にした。
- wall-clockは中央値を使い、機能成否のassertionと性能値を分離した。
- 主観的な音質scoreを作らず、設計入力のmusic weightとpolyphonyを提示した。
- 100 iterations上限で誤操作による過剰実行を防いだ。

## 影響範囲

native Rust benchmark/exampleと通常test時間。codec wire format、WASM、Web UI、API、DBに変更はない。

## 追加・更新したテスト

- 全10経路のclean round-trip、WAV metrics、5 channel case schema。
- iterations 0および101の拒否。

## 実行した確認コマンド

- `cargo fmt --all`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test benchmark::tests --locked`: 2件成功
- `cargo test --all-targets --all-features --locked`: 171件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `cargo run --release --locked --example v2_benchmark -- 128 3`: 成功

測定環境はApple Silicon arm64、Rust 1.98.0、release profile。128-byte fixtureは全経路clean round-trip成功。Quietは20,100 ms / 50.95 bps、Balancedは119,440 ms / 8.57 bps、FixedFallbackは386,320 ms / 2.65 bps。Secure Balanced中央値はencode 75,094 µs、decode 261,834 µs。5 channel条件は全profile 2/5成功で、アナログsymbol判定の改善余地を示した。

## CIで確認される内容

新しいbenchmark unit testとexample buildを含むRust fmt、Clippy、test、check、build、audit。既存Web全品質ゲート。

## 未解決の課題

- 32/256-byteなどfixture size別reportと回帰比較。
- Opus round-trip fixture。
- 実smartphone recording corpus。
- channel成功率が全profile同値になるアナログdecoder限界の分析。

## 次にやること

fixture size別runnerとreport比較を追加し、次にOpus fixtureの再現可能な生成方法を決める。

## 次回最初に見るべきファイル

`packages/harmonic-core/src/benchmark.rs`、`docs/benchmarks/2026-09-10-v2-text-128.json`、`docs/adr/0012-benchmark-metrics.md`。

## 引き継ぎ事項

wall-clock値を異なるhardware間のpass/fail閾値にしない。`music_weight`は音質評価値ではなくprofile設計値。実録音なしでsmartphone耐性を達成済みと扱わない。
