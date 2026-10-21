# 作業報告書

## 作業日時

2026年10月21日 15時23分19秒（作業環境の現在時刻、UTC）

## 作業対象・目的

Node.js 2026年10月セキュリティリリース（Brotli解凍時のメモリ破壊、TLS証明書検証の回避など、最高深刻度High）への対応。CIで使うNode.jsを修正版に固定する。修正版は https://nodejs.org/dist/index.json で `v22.23.4`（2026-10-20、`security: true`）を確認した。

## 変更内容・変更したファイル

- `.node-version`（新規、`22.23.4`）
- `.github/workflows/ci.yml`（`web` / `deploy` ジョブ）: `node-version: 22` → `node-version-file: .node-version`

## 設計上の意図

`setup-node` にメジャー版だけを渡すと、ランナーのtoolcacheにある古い22.xが使われることがある。修正版を明示し、版の指定を `.node-version` 1か所にまとめる（nvm・fnm・Volta等のローカル版管理も同じファイルを読める）。

## 影響範囲

CIとVercelへのprebuilt deploy時のNode.js。Webは静的SPAで、本番でNode.jsがサーバーとして動く箇所はない。

## 追加・更新したテスト・確認コマンド

Node.js 22.23.4で `apps/web` にて `npm ci && npm run format:check && npm run lint && npm run typecheck && npm test && npm run build` → すべて成功。Playwright E2Eはローカル未実行（CIで確認）。

## CIで確認される内容

`rust` / `web` ジョブ、main pushの `deploy` ジョブ。

## 未解決の課題・次にやること

1. ローカル開発環境の `node -v` を22.23.4以上へ更新する（ユーザー作業）。
2. 次回のNode.jsセキュリティリリースでは `.node-version` を更新する。

## 次回最初に見るべきファイル

`.node-version`、`.github/workflows/`

## 引き継ぎ事項

`npm audit` は依存パッケージの検査で、Node.js本体の版は検査しない。
