# 作業報告書

## 作業日時

2026年09月10日 21時23分28秒

## 作業対象

v2 Phase 9 Benchmarkの共通telemetry schemaと決定論的channel計測。

## 作業目的

Phase 9以降の通信路をFinal Recovery Rate、Raw Symbol Accuracy、Corrected Errors、Payload Bitrate、Decode Time、Playback Durationの6指標で比較できるようにする。

## 変更内容

- benchmark report schemaをversion 2にし、3つの復元telemetry fieldを必須化した。
- clean packetとchannel後のFEC前packetをbit単位で比較するようにした。
- raw packetを返せないcaseを0%とする保守的な集計にした。
- raw errorがあり、かつ最終Payloadが完全一致したbitだけをCorrected Errorsに数えた。
- MIDI計測を別moduleへ分離し、全ファイルを300行以下に保った。
- 128-byte baseline JSONをschema v2へ更新した。

## 変更したファイル

- `packages/harmonic-core/src/benchmark.rs`
- `packages/harmonic-core/src/benchmark/channel.rs`
- `packages/harmonic-core/src/benchmark/midi.rs`
- `packages/harmonic-core/src/v2/adaptive_decode.rs`
- `docs/benchmarks/2026-09-10-v2-text-128.json`
- `docs/adr/0012-benchmark-metrics.md`、`docs/TODO.md`

## 変更意図

成功/失敗だけでなくsymbol層とFEC後の差を同じreportで比較し、改善すべき層を判断できるようにするため。

## 設計上の意図

telemetryはbenchmark層に限定し、本番decoderのwire formatや公開APIを変えない。`decode_profile_packet`はcrate内だけに公開し、UI/WASMへは露出させない。

## 影響範囲

Rust benchmarkとJSON schema。codecの通常復号結果、Web UI、API、DBに変更はない。

## 追加・更新したテスト

- packet bit error数の正常系。
- 完全一致、1-bit誤りから最終復元成功、raw decode失敗を含む集計境界値。
- schema versionと6指標の範囲。

## 実行した確認コマンド

- `cargo test benchmark:: --locked`: 4件成功。
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功。
- `cargo run --release --locked --example v2_benchmark -- 32 1`: 成功。
- `cargo run --release --locked --example v2_benchmark -- 128 3`: 成功。
- Rust fmt / Clippy / test 173件 / check / build: 成功。
- Web側は直前のOpusコミットでtypecheck / E2E typecheck / format / lint / Vitest 42件 / build / Playwright 20件 / k6 / npm auditを確認し、この変更ではWeb codeを変更していない。

## CIで確認される内容

Rust fmt、Clippy、unit/all-target test、check、build、audit。Webの全品質ゲート、Playwright、k6、npm audit、成功後のVercel gate。

## 未解決の課題

- 現行5 channelでは全Acoustic profileがFinal Recovery 40%、Raw Symbol Accuracy 40%、Corrected Errors 0である。失敗はpacket内bit誤りより前のsymbol/timing detectorで起き、FECへ到達していない。
- 部分symbol列を返すdiagnostic decoderは未実装。現行Raw Accuracyはraw packet復元失敗caseを0%とする。
- Secure benchmarkはAEAD salt/nonceのランダム性により出力音長が完全に決定論的ではない。回帰比較では許容幅が必要。

## 次にやること

32/128/256-byteのfixture size別reportと、機能値は厳密、wall-clock/Secure音長は許容幅を持つregression comparisonを追加する。

## 次回最初に見るべきファイル

`packages/harmonic-core/src/benchmark.rs`、`packages/harmonic-core/src/benchmark/channel.rs`、`docs/benchmarks/2026-09-10-v2-text-128.json`。

## 引き継ぎ事項

Raw Symbol Accuracyの定義を変える場合はschema versionを上げる。失敗caseを分母から除外しない。実端末の失敗条件もreportから削除しない。
