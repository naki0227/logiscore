# 作業報告書

## 作業日時

2026年09月10日 09時53分23秒

## 作業対象

v2 Phase 9のOpus round-trip fixture、Rust音響decoder、Web Audio import E2E。

## 作業目的

損失圧縮されたOpus/OggをブラウザがPCM化し、v2 Text payloadへ自動復元できることを再現可能なfixtureで検証する。

## 変更内容

- `FixedFallback`でText fixtureを生成するRust exampleを追加した。
- ffmpeg/libopusで128 kbps CBR、low-delay、5 ms frameのOgg fixtureを生成した。
- OpusをPCM WAVへ戻してnative decoderを確認するexampleを追加した。
- Playwright Chromiumからfixtureをuploadし、Web Audio経由で`Opus`へ復元するE2Eを追加した。
- fixture再生成コマンドと制約を文書化した。

## 変更したファイル

- `packages/harmonic-core/examples/generate_recording_fixture.rs`
- `packages/harmonic-core/examples/decode_recording_fixture.rs`
- `apps/web/e2e/fixtures/logiscore-opus.ogg`
- `apps/web/e2e/fixtures/README.md`
- `apps/web/e2e/recording-formats.spec.ts`
- `README.md`、`docs/adr/0012-benchmark-metrics.md`、`docs/TODO.md`

## 変更意図

実装したWeb Audio変換をモックせず、実ブラウザと実Opus bitstreamで回帰を検出するため。

## 設計上の意図

Opus変換でrhythmic profileの音長境界がプリエコーにより変形することを切り分けた。閾値を無根拠に緩めず、既存設計で圧縮チャネルに強い単音・固定slotの`FixedFallback`をfixtureに採用した。

## 影響範囲

QA example、E2E fixture、自動ブラウザテスト、ドキュメント。wire format、本番API、DB、UIの仕様変更はない。

## 追加・更新したテスト

- Playwright: Opus/Ogg uploadからv2 Textの完全一致復元。
- native verification: Opus→48 kHz float PCM WAV→Rust adaptive decoderの完全一致復元。

## 実行した確認コマンド

- fixture生成用`cargo run --release --locked --example generate_recording_fixture`: 成功。
- `ffmpeg` Opus encode/decode: 成功。
- `cargo run --release --locked --example decode_recording_fixture`: `Opus`を復元。
- `npm run test:e2e -- recording-formats.spec.ts`: Chromium 1件成功。
- Rust fmt / Clippy / test 171件 / check / build: 成功。
- `cargo audit`: 126 dependencies、脆弱性なし。
- Web typecheck / E2E typecheck / format / lint / Vitest 42件 / build: 成功。
- `npm audit --audit-level=high`: 脆弱性0件。
- `npm run test:e2e`: Chromium 20件成功。
- k6 2.2.0: 5 VU、312 iterations、1,250 requests、checks 100%、failure 0%、p95 19.18 ms。

## CIで確認される内容

Rust fmt、Clippy、171+件のtest、check、build、cargo-audit。Web typecheck、format、lint、unit test、build、Playwright、k6、npm audit。

## 未解決の課題

- rhythmic acoustic profileはOpus後の音長境界検出に失敗する。
- 実smartphoneのマイク・スピーカー往復corpusは未収集。

## 次にやること

fixture size別benchmark reportと回帰比較を追加する。rhythmic Opus対応はtone-boundary検出の専用設計・再現テスとして分離する。

## 次回最初に見るべきファイル

`packages/harmonic-core/src/benchmark.rs`、`docs/benchmarks/2026-09-10-v2-text-128.json`、`docs/TODO.md`。

## 引き継ぎ事項

Opus fixtureの生成条件を変える場合はRust native復元とPlaywright Web Audio復元を両方実行する。実端末corpusなしでsmartphone対応済みと扱わない。

Phase 9以降の「できた」は6共通指標を持つreportで判定する。任意位置/ループ、実環境、codec matrix、WebRTC/Meet、電話の順に通信路を段階的に厳しくする。Visual Codecはv2完成後のv3とする。
