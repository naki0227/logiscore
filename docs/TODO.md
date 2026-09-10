# Logiscore Todo

最終更新: 2026-09-10

## 進行中

- [進行中] v2 Phase 9: Benchmark
  - [完了] 決定論的fixtureとJSON report schema
  - [完了] Dense / Rhythmic / 全Acoustic profile / Secureの速度・size・bitrate計測
  - [完了] 5段階の決定論的channel recovery計測
  - [完了] 全channel reportを6共通指標へ統一
    - [完了] Payload bitrate / Decode time / Playback duration
    - [完了] Final Recovery Rate / Raw Symbol Accuracy / Corrected Errorsのtelemetry
  - [完了] 32/128/256-byte fixture reportとfunctional regression比較
  - [完了] FixedFallbackのOpus round-trip fixture / Rust・Playwright復元
  - [進行中] smartphone recording corpus
    - [完了] manifest schema、収録手順、planned/captured/verified validation
    - [完了] captured M4Aを自動復元するPlaywrightシナリオ
    - [未着手] MacBook Speaker→1m→iPhone Voice Memosの実M4A収録
    - [未着手] 実録音の6共通指標report

## 未着手

- [未着手] 701KB超のWeb bundleをcode splittingする
- [未着手] rhythmic acoustic profileのOpus後プリエコーに耐えるtone-boundary検出を設計する
- [未着手] v2 field validation 2: MacBook→1m→iPhone Voice Memos→M4AのText/File/Project corpus
- [未着手] v2 field validation 3: 任意位置・ループ録音のchunk/checkpoint protocol
- [未着手] v2 field validation 4: 会話・TV・音楽・Café・残響の実環境matrix
- [未着手] v2 field validation 5: PCM/Opus/AAC/MP3/resampling codec matrix
- [未着手] v2 field validation 6: WebRTC後にGoogle Meet通信
- [未着手] v2 field validation 7: VoiceCallProfileと電話回線
- [未着手] v2 field validation 8: v2公開デモ3本とREADME更新

## 要確認

- [要確認] GitHub Secrets `VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID`を登録し、Repository Variable `VERCEL_DEPLOY_ENABLED=true`で本番deployを有効化する
- [要確認] GitHub ActionsのNode.js 20 runtime廃止warningに対応するaction majorが利用可能になったら更新する

## 技術的負債

- [完了] `apps/web/src/App.tsx` を960行から294行へ分割する
- [要確認] `apps/web/src/hooks/useAudioEngine.ts` が323行のため、音源定義と再生schedulerを分離する
- [要確認] Rustの`protocol/midi_gen.rs`、`protocol/mod.rs`が300行を超えているため、v1互換境界を分割する
- [完了] Rustの`lib.rs`からv2 WASM境界と密度計算を分離し、297行へ縮小する
- [要確認] Rustの手動debug binaryは自動ビルド対象外。必要ならexamplesまたはintegration testsへ整理する

## 完了

- [完了] Rust/Web CI成功後だけ実行可能なVercel production deploy jobを追加
- [完了] CI runner Rust 1.98.1で追加されたClippy lint 3件へ適合
- [完了] v2 Phase 8: Secure Mode
  - [完了] Argon2id / ChaCha20-Poly1305 / CSPRNGの依存選定とADR
  - [完了] versioned Secure Envelopeと認証失敗policy
  - [完了] secure envelopeをv2 packet / adaptive WAVへ統合
  - [完了] Text / Source File / ProjectのWASM境界とPassword UX
  - [完了] UTF-8 byte上限・確認一致・browser storage非保存
  - [完了] Playwrightによる正誤Password・非開示・randomization・全payload往復
  - [完了] Cargo/npm依存監査と既知advisory修正版へのlock更新

- [完了] v2 Phase 7: Adaptive Profiles
  - [完了] Environment / CalibrationMetrics domain
  - [完了] deterministic profile selectorとconfidence
  - [完了] fallback orderとduration estimator
  - [完了] profile timingとFEC 0/1/2/3のPCM適用
  - [完了] calibration-first multi-profile auto decoderと旧WAV互換
  - [完了] monophonic FixedFallback codec
  - [完了] WASM / Environment UI / Priority / duration / confidence
  - [完了] 全EnvironmentのPlaywright browser round-trip

- [完了] v2 Phase 6: Physical Acoustic
  - [完了] PCM16 / float32 WAV encode/decode
  - [完了] stereo downmixと8〜384 kHz入力検証
  - [完了] sample-rate normalizationとWASM 32-bit overflow対策
  - [完了] gain/noise/clipping/offset/echo channel simulation
  - [完了] Reliable WAVの生成・再生・download・録音file import UI
  - [完了] WAV・Web Audio PCM・破損録音のPlaywright自動シナリオ
  - [完了] Vitest 4.1.11とDOMPurify 3.4.15へ更新しnpm audit 0件
- [完了] v2 Phase 5: Error Correction
  - [完了] CRC-32
  - [完了] Hamming SECDED (13,8)
  - [完了] 8-codeword bit Interleaving
  - [完了] 3-copy bit-majority Repetition
  - [完了] FEC profile `1` packet / Reliable PCM API
  - [完了] 訂正・検出・burst・property tests
  - [完了] Playwrightで全Reliable PCM payloadをブラウザ内round-trip
- [完了] v2 Phase 4: Audio Decoder
  - [完了] 8 kHz clean PCM generation
  - [完了] 2-tone preambleと32-bit packet length framing
  - [完了] Goertzelによる4声symbol detection
  - [完了] onset/duration timing recovery
  - [完了] leading offset・low noise・不正入力のテスト
  - [完了] Text / Source File / ProjectのRust・WASM PCM API
  - [完了] Playwrightで全PCM payloadをブラウザ内round-trip

- [完了] v2 Phase 3: Musical Density
  - [完了] Rhythm Encoding（4 durations）
  - [完了] Pitch / RhythmのMultiple Symbol Channels
  - [完了] 強拍8候補・弱拍32候補のDynamic Candidate Count
  - [完了] 平均6 bit/eventへのbitrate最適化
  - [完了] codec profile `2`のRhythmic Musical MIDI
  - [完了] WASM / Web / Playwright全payload round-trip
- [完了] v2 Phase 2: Musical Baseline
  - [完了] Tonal Context / Chord Planner
  - [完了] Candidate Generator / 4 Voice Architecture
  - [完了] Voice Leading / Consonance Scoring
  - [完了] 4-bit Symbol streamの可逆domain codec
  - [完了] Musical MIDI Transport / WASM / Web UI
  - [完了] PlaywrightによるText / Source File / Project Musical round-trip
- [完了] v2 Phase 1: Protocol Cleanup
  - [完了] Payload Type定義
  - [完了] Text PayloadのRust API
  - [完了] 7-byte Binary Header
  - [完了] Transport abstraction
  - [完了] Existing Dense Mode adapter
  - [完了] v2 Text APIのWASM公開とWeb UI接続
  - [完了] Source File canonical format
  - [完了] Project canonical archive format
- [完了] Frontendのcodec routing・file type unit testを追加する
- [完了] Playwright ChromiumでText/Source File/Project round-trip E2Eを追加する
- [完了] k6で静的アセット配信の負荷シナリオと閾値を追加する
- [完了] GitHub Actions CIを追加（Rust/Webのformat、lint、typecheck、test、check、build、npm audit）
- [完了] 既存Rustコードをrustfmt適合させ、Clippy警告を解消
- [完了] WebへPrettier/Vitest scriptsを追加
- [完了] `index.html` の閉じていないhead要素を修正
- [完了] npm high severity脆弱性を互換範囲内で解消
- [完了] Vite 8 native config loader向けに`import.meta.dirname`へ移行

## 次回最初に着手するタスク

実smartphone recording corpusの受入schemaと収録手順を設計する。最初に `docs/logiscore_improvement_design.md` のLevel 4 / v2フィールド検証roadmap、`docs/TODO.md`、`packages/harmonic-core/src/benchmark.rs` を読む。最初のコマンドは `cargo test --all-targets --all-features --locked` と `npm run test:e2e`。
