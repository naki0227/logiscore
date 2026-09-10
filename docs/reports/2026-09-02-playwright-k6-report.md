# 作業報告書

## 作業日時

2026年09月02日 09時48分37秒 JST

## 作業対象

WebのPlaywright E2E、k6静的アセット負荷シナリオ、CI統合。

## 作業目的

unit testとWASM単体round-tripだけでなく、実ブラウザ上のユーザーフローと配信負荷を自動検証する。

## 変更内容

- Playwright Test 1.62.1と対応Chromiumを導入した。
- TextとSource Fileについて、入力、WASM encode、MIDI download、再import、復元までをE2E化した。
- Projectがv1表示であることと、未実装ModeがdisabledであることをE2E化した。
- Given/When/Thenを`test.step`で表現し、Cucumber依存なしでシナリオを可読化した。
- 失敗時のtrace、screenshot、video、HTML reportを設定した。
- VitestとPlaywrightのtest探索範囲を分離した。
- k6 2.2.0でHTML、JS、CSS、WASMを取得する5 VU段階負荷シナリオを追加した。
- Playwrightとk6をGitHub Actionsへ追加した。
- autocannonは依存経路にmoderate脆弱性が増えたため採用しなかった。

## 変更したファイル

- `apps/web/playwright.config.ts`
- `apps/web/tsconfig.e2e.json`
- `apps/web/vitest.config.ts`
- `apps/web/e2e/codec-roundtrip.spec.ts`
- `apps/web/tests/load/static-assets.js`
- `apps/web/package.json`
- `apps/web/package-lock.json`
- `apps/web/.prettierignore`
- `apps/web/src/features/workspace/AppHeader.tsx`
- `apps/web/src/features/workspace/SourcePanel.tsx`
- `apps/web/src/features/workspace/OutputPanel.tsx`
- `.github/workflows/ci.yml`
- `.gitignore`
- `README.md`
- `docs/TODO.md`

## 変更意図

ブラウザAPI、download、file upload、WASM初期化を含む統合経路はunit testだけでは保証できないため、Chromiumで利用者と同じ操作を実行する。k6はcodec性能ではなく静的アセット配信の回帰検出に限定する。

## 設計上の意図

Playwright標準runnerへ集約し、追加のCucumber step定義層を持たない。locatorはroleとaccessible nameを優先した。CIは安定性のためworker 1、失敗時のみ重い成果物を保持する。k6はアプリ依存へ追加せず、version固定した公式containerで実行する。

## 影響範囲

Webテスト、CI時間、status/editor/import controlのaccessibility属性。codec、protocol、DB、HTTP API、認証、永続化の振る舞いは変更していない。

## 追加・更新したテスト

- v2 Text Unicode MIDI download/import round-trip
- v2 Source File metadata/content MIDI download/import round-trip
- Project v1表示とMusic/Reliable disabled状態
- k6によるHTML、JS、CSS、WASM配信シナリオ
- Playwright E2E 3件、Frontend unit test 16件、Rust unit test 71件

## 実行した確認コマンド

- `npx playwright install chromium`: 成功
- `npm run test:e2e`: Chromium 3件成功
- `npm run typecheck:e2e`: 成功
- k6 2.2.0 `tests/load/static-assets.js`: 成功
  - 5 VU、9秒、332 iterations、1,330 HTTP requests
  - checks 100%、request failure 0%、p95 9.94ms
- `cargo fmt --all -- --check`: 成功
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: 成功
- `cargo test --all-targets --all-features --locked`: 71件成功
- `cargo check --all-targets --all-features --locked`: 成功
- `cargo build --all-targets --all-features --locked`: 成功
- `npm run typecheck`: 成功
- `npm run format:check`: 成功
- `npm run lint`: warning 0件で成功
- `npm test`: 16件成功
- `npm run build`: 成功
- `npm audit --audit-level=high`: 成功（high/critical 0件）
- `git diff --check`: 成功

## CIで確認される内容

従来のRust/Web品質ゲートに加え、Playwright Chromium E2E 3件とk6静的アセット負荷閾値を確認する。失敗時もPlaywright HTML reportを14日間artifactとして保持する。GitHub上は未pushのため未実行。

## 未解決の課題

- k6結果はローカルVite previewの値であり、本番CDNの容量評価ではない。
- ローカルDocker/OrbStack daemonが停止中だったため、k6は公式release binaryを一時取得して実行した。
- Projectのfolder drag-and-drop E2Eは、ブラウザ固有Directory Entry APIのfixture設計が未実装。
- Web bundleが約686KBでcode splitting警告が残る。
- DOMPurify/Monaco由来のmoderate 1件・low 1件が残る。

## 次にやること

Project v2 archive schemaを決定する。実装後、複数ファイルのencode/import E2Eを追加し、本番URLが確定したらk6の別profileでCDN smokeを行う。

## 次回最初に見るべきファイル

- `apps/web/e2e/codec-roundtrip.spec.ts`
- `apps/web/tests/load/static-assets.js`
- `.github/workflows/ci.yml`
- `packages/harmonic-core/src/payload.rs`
- `docs/TODO.md`

## 引き継ぎ事項

次回最初に`npm run test:e2e`、`npm test`、`cargo test --all-targets --all-features --locked`を実行する。E2E locatorはCSS classよりrole/labelを優先する。k6のlocalhost数値を本番性能として扱わない。負荷閾値変更は対象環境と根拠を報告書へ残す。
