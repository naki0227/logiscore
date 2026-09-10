# 作業報告書

## 作業日時

2026年09月09日 19時49分55秒 JST

## 作業対象

v2 Phase 7 Adaptive Profilesのdomain設計と初期実装。

## 作業目的

environment、calibration、payload size、reliability priorityから再現可能なAcoustic Profile、confidence、fallback、estimated durationを決める基盤を作る。

## 変更内容

- Auto/Quiet/Conversation/Noisy/Online/LongDistance environmentを追加した。
- noise floor、SNR、clipping、reverberationのvalidated calibration valueを追加した。
- 7種類の固定Acoustic Profile IDとparameter tableを追加した。
- explicit/Auto environment、priority、payload sizeによるselectorを追加した。
- profile別fallback順とduration estimatorを追加した。
- 正常系、境界、異常値、duration monotonicityのunit testsを追加した。

## 変更したファイル

- `packages/harmonic-core/src/adaptive/mod.rs`
- `packages/harmonic-core/src/adaptive/environment.rs`
- `packages/harmonic-core/src/adaptive/profile.rs`
- `packages/harmonic-core/src/adaptive/selector.rs`
- `packages/harmonic-core/src/adaptive/estimate.rs`
- `packages/harmonic-core/src/adaptive/tests.rs`
- `packages/harmonic-core/src/error.rs`
- `packages/harmonic-core/src/lib.rs`
- `docs/adr/0009-adaptive-profile-selection.md`
- `docs/TODO.md`

## 変更意図

profile選択をUI条件分岐やaudio decoderへ直接埋め込まず、決定的にテストできるdomainとして分離するため。

## 設計上の意図

外部serviceやML modelへ依存せず、固定tableとvalidated metricsでsender/receiverが同じ判断を再構築できる構造にした。全追加fileは300行以内で、新規依存はない。

## 影響範囲

現時点ではdomain API追加のみ。既存PCM/WAVのwire behaviorとWeb UIはまだ変更していない。

## 追加・更新したテスト

- explicit environmentからstable profileへのmapping
- calibrationによるAuto environment判定
- reliability priorityとlarge payloadによるprofile調整
- fallback末尾のFixedFallback保証
- payload/profileに対するduration増加
- 不正calibration/priority拒否

## 実行した確認コマンド

- `cargo fmt --all`: 成功
- `cargo test adaptive --all-features`: 5件成功
- `cargo clippy --all-targets --all-features -- -D warnings`: 成功

Phase 6確定時にはRust 147件、Web 27件、Playwright 14件、k6、build、npm audit 0件を確認済み。Phase 7全ゲートは統合後に再実行する。

## CIで確認される内容

Rust format、Clippy、test、check、build。Web typecheck、E2E typecheck、format、lint、unit test、build、Playwright Chromium、k6、npm audit。

## 未解決の課題

- profile timing/FEC parameterを実際のPCM/WAVへ適用していない。
- calibration signal解析とauto decoder retryは未実装。
- FixedFallback codecは未実装。
- Environment UIとPlaywright scenarioは未実装。

## 次にやること

PcmProfileへtiming倍率を安全に適用し、environment別WAV encodeと複数profile decodeを追加する。

## 次回最初に見るべきファイル

- `packages/harmonic-core/src/adaptive/`
- `packages/harmonic-core/src/audio/profile.rs`
- `packages/harmonic-core/src/v2/wav.rs`
- `docs/adr/0009-adaptive-profile-selection.md`
- `docs/TODO.md`

## 引き継ぎ事項

AcousticProfile IDとparameter tableは将来wire互換になるため、適用後の変更はversioningを伴う。既存Balanced WAVはdecode候補として維持する。ブラウザ変更時はPlaywright scriptを追加し自動実行する。
