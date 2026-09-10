# ADR 0011: Vercel本番deployをCI成功後に限定する

- Status: Accepted
- Date: 2026-09-10

## 背景

mainへのpush後、品質ゲートを通過した成果物だけをVercel本番環境へ反映したい。現在Vercel用GitHub Secretsは未登録である。

## 課題

- Rust/Web両方のCI成功をdeployの前提にする。
- token、organization ID、project IDをリポジトリへ保存しない。
- 認証情報が未設定の間は既存CIを失敗させない。
- monorepo内の`apps/web`を明示的なdeploy対象にする。

## 選択肢

1. Vercel Git Integrationへ任せ、CIと並行してdeployする。
2. CI workflow内の独立jobからVercel CLIでdeployする。
3. CIと無関係な手動deployだけを使う。

## 採用した案

選択肢2を採用した。`deploy` jobは`needs: [rust, web]`、mainへのpush、Repository Variable `VERCEL_DEPLOY_ENABLED=true`をすべて満たす場合だけ動く。Vercel CLIは59.15.0へ固定し、`pull -> build --prod -> deploy --prebuilt --prod`を`apps/web`で実行する。

## 採用理由

CI failure時の本番反映を構造的に防ぎ、Vercel公式のcustom workflow手順に沿ってbuild artifactをdeployできる。Variableによる明示的な有効化で、Secrets未設定状態を安全に扱える。

## メリット

- Rust/Webの全品質ゲート成功が本番deployの必須条件になる。
- Vercel認証情報をGitHub Secretsに限定できる。
- CLI version固定によりCIの再現性を確保できる。
- GitHub Environment上で本番URLとdeploy履歴を追跡できる。

## デメリット

- Vercel Git Integrationと併用すると二重deployになるため、導入時に片方へ統一する必要がある。
- CLI更新は手動で監査・検証する必要がある。
- GitHub Actionsの実行時間とVercel upload時間が増える。

## 将来的な見直し条件

- Preview deploymentやapproval gateが必要になった場合。
- Vercel公式workflow/CLIの推奨手順が変わった場合。
- deploy先を別hostingへ変更する場合。
