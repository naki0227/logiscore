# 作業報告書

## 作業日時

2026年09月12日 10時36分38秒

## 作業対象

v2 Phase 9 Benchmarkの実iPhone録音baselineと全品質ゲート。

## 作業目的

提供された無加工M4Aを再現可能なcorpusへ保存し、MacBook Speaker→空気1m→iPhone Voice Memos→AAC→Web Audio→Logiscoreの完全復元を自動テストで証明する。

## 変更内容

- iPhone録音原本をSHA-256付きでverified corpusへ登録した。
- 実録音の弱いpayload symbolをFECへ渡せるようFixedFallbackの絶対energy floorを調整した。
- captured録音は失敗も測定し、verified録音だけ完全一致を要求する状態規則へ修正した。
- AAC primingの2 sample差を許容し、復元・精度指標は厳密一致のまま維持した。
- Phase 9 Benchmarkを完了へ更新した。

## 変更したファイル

詳細な一覧は`2026-09-10-recording-telemetry-report.md`を参照。主対象はRust recording telemetry、WASM、Web measurement adapter、Playwright corpus、実M4A/WAV、ADR、設計書、TODO。

## 変更意図

実録音でraw packetが3 bitだけ誤っていたにもかかわらず、弱い1 symbolの絶対floor判定でFEC前に全体を捨てていた。同期の相対周波数判定は維持し、誤り訂正可能なraw packetを後段へ渡すために修正した。

## 設計上の意図

FixedFallbackは物理伝送の保守的baselineであり、音楽性を完成形とはしない。計測はRust、ファイルdecodeはWeb Audio、ブラウザシナリオはPlaywrightへ分離した。原本hashによりfixture改変を検出する。

## 影響範囲

FixedFallback decoder、benchmark WASM API、E2E corpus。wire format、DB、外部API、production UIに破壊的変更はない。

## 追加・更新したテスト

- 弱いpayload symbolのRust regression test。
- 実M4Aのhash、Web Audio import、Text完全一致、6指標比較。
- 失敗captured recordingも測定可能なPlaywright状態規則。
- AAC priming差を1 ms未満だけ許容する再生時間比較。

## 実行した確認コマンド

- Rust format / clippy / 180 tests / check / build: 成功。
- Cargo audit: 126依存、既知脆弱性なし。
- Web typecheck / E2E typecheck / format / lint / Vitest 44件 / build: 成功。
- Playwright Chromium: 25件成功。
- npm audit: 既知脆弱性0件。
- k6: 650/650 checks、HTTP failure 0%、p95 13.23 ms。

## CIで確認される内容

RustとWebの上記品質ゲート、Playwright、k6、依存監査。両品質job成功後にVercel deploy gateを評価する。

## 未解決の課題

- FixedFallbackは約136.72秒、実録音約139.71秒、約1.20 bpsで遅い。
- 単音固定slotのためシンセ的で不気味に聞こえ、目標とする現代的な音楽性は未達。
- Source File / Projectの実録音、3m/5m、別端末は未検証。
- production JS 706.82 kBにcode splitting警告がある。

## 次にやること

任意位置・ループ録音用のcheckpoint protocolをADRとdomain testから設計する。並行する物理検証はSource File、Projectの順に拡張する。

## 次回最初に見るべきファイル

`docs/logiscore_improvement_design.md`、`docs/TODO.md`、`packages/harmonic-core/src/audio/fixed.rs`、`apps/web/e2e/recording-corpus.spec.ts`。

## 引き継ぎ事項

ブラウザ確認はPlaywrightのみを使う。実録音原本を加工しない。FixedFallbackの音色改善と通信成立を混同せず、musical profileは同じ6指標で比較して採用する。
