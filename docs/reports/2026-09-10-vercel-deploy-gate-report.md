# 作業報告書

## 作業日時

2026年09月10日 09時21分39秒

## 作業対象

GitHub Actions、Vercel production deployment。

## 作業目的

Rust/Web CI成功後に限り、mainのWeb成果物をVercelへ本番deployできる経路を用意する。

## 変更内容

`needs: [rust, web]`を持つVercel production jobを追加した。main pushと`VERCEL_DEPLOY_ENABLED=true`を有効化条件とし、Vercel CLI 59.15.0でproduction環境取得、build、prebuilt deployを行う。

## 変更したファイル

- `.github/workflows/ci.yml`
- `docs/adr/0011-vercel-deployment-gate.md`
- `docs/TODO.md`

## 変更意図

テスト失敗中のcommitを本番へ出さず、Secrets未設定の現状でもCIを壊さないため。

## 設計上の意図

deployをbuild/test jobから分離し、依存関係で品質gateを表現した。tokenとproject識別子はGitHub Secretsのみから注入する。

## 影響範囲

main push時のGitHub Actions。本体コード、API、DB、既存テストへの変更はない。

## 追加・更新したテスト

アプリテストの追加はない。workflow YAML構文とjob依存関係を確認する。

## 実行した確認コマンド

- `gh secret list`: Vercel Secrets未登録
- `gh variable list`: deploy有効化Variable未登録
- YAML parserによるworkflow構文確認: 成功

## CIで確認される内容

Rust/Web全品質ゲート成功後のみVercel deploy jobが評価される。未設定中はdeploy jobをskipする。

## 未解決の課題

GitHub Secrets 3件の登録、Vercel project link、二重deployを避けるためのVercel Git Integration方針決定が必要。

## 次にやること

`VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID`をSecretsへ登録し、`VERCEL_DEPLOY_ENABLED=true`を設定して初回deployを確認する。

## 次回最初に見るべきファイル

`.github/workflows/ci.yml`、`docs/adr/0011-vercel-deployment-gate.md`。

## 引き継ぎ事項

秘密値をログ、コミット、Variableへ保存しない。既存Vercel Git Integrationが有効なら二重deployを避けるため無効化してからVariableを有効にする。
