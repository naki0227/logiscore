# 作業報告書

## 作業日時

2026年09月10日 21時43分27秒

## 作業対象

v2 Phase 9の実smartphone recording corpus受入基盤。

## 作業目的

合成音声を実録音と混同せず、MacBook Speaker→1m→iPhone Voice Memos→M4A→Logiscoreの固定条件を再現可能なmetadataとPlaywrightで検証できるようにする。

## 変更内容

- corpus schema version 1と最初のText planned entryを追加した。
- payload SHA-256、収録距離、speaker、recorder app、環境、端末、日時、ファイル、6指標measurementを定義した。
- planned / captured / verifiedの状態遷移と必須fieldをruntime validationする。
- captured/verified entryをPlaywrightが実ファイルuploadし、元Textとの完全一致を自動検証する。
- 収録・登録・検証手順を追加した。

## 変更したファイル

- `apps/web/e2e/fixtures/recordings/corpus.json`
- `apps/web/e2e/fixtures/recordings/README.md`
- `apps/web/e2e/recording-corpus.ts`
- `apps/web/e2e/recording-corpus.spec.ts`
- `docs/TODO.md`

## 変更意図

実録音が存在しない段階を`planned`として明示し、ファイルなしで「空気を通して戻った」と誤って完了扱いしないため。

## 設計上の意図

`captured`では物理metadataとM4Aを必須にして自動E2Eを実行する。`verified`ではさらに6指標measurementを必須にする。失敗録音も削除せず限界測定へ使う。

## 影響範囲

Playwright E2E、fixture metadata、収録運用。production Web UI、codec、wire format、API、DBに変更はない。

## 追加・更新したテスト

- planned 1m Text manifestの正常系。
- capturedなのに端末・日時・ファイルがないmanifestの異常系。
- captured/verified entryごとのWeb Audio importとText完全一致（実ファイル追加時に自動生成）。

## 実行した確認コマンド

- `npm run typecheck:e2e`: 成功。
- `npm run lint`: 成功。
- `npm run format:check`: 成功。
- `npm run test:e2e`: corpus正常・異常系を含むChromium 22件成功。
- Web typecheck / E2E typecheck / format / lint / Vitest 42件 / build: 成功。
- Rust code・依存変更なし。直前commitのRust 175件とCI成功を継承し、push後CIで再確認する。
- GitHub CI run `34478610203`: 全job成功。ただしReliable Project WAV復元が初回の5秒timeoutを超えretry成功したため、当該重い復号assertionだけ15秒へ明示し再検証する。

## CIで確認される内容

Web typecheck、E2E typecheck、format、lint、Vitest、build、Playwright全件、k6、npm audit。Rust全ゲート。成功後のVercel gate。

## 未解決の課題

- 実iPhone M4Aはまだ存在しないため、物理round-tripは未達。
- 実録音PCMからRaw Symbol AccuracyとCorrected Errorsを出すbrowser telemetry境界は未実装。
- 端末model、部屋、音量などの実測metadataは収録時に確定する。
- GitHub ActionsのNode.js 20 action runtime廃止warningに対し、対応版actionへの更新確認が必要。

## 次にやること

指定条件で実M4Aを収録し、manifestをcapturedへ変更してPlaywrightを実行する。その後、実録音用6指標report generatorを接続する。

## 次回最初に見るべきファイル

`apps/web/e2e/fixtures/recordings/README.md`、`apps/web/e2e/fixtures/recordings/corpus.json`、`apps/web/e2e/recording-corpus.spec.ts`。

## 引き継ぎ事項

実録音は変換前の原本を保存する。手動ブラウザ確認を完了条件にしない。実M4Aがないままentryをcaptured/verifiedにしない。失敗fixtureも削除しない。
