# ADR 0012: Phase 9測定を決定論的fixtureとJSON reportで行う

- Status: Accepted
- Date: 2026-09-10

## 背景

Phase 9ではMusic Quality、Bitrate、Playback Time、Noise Robustness、Distanceのトレードオフを定量化する必要がある。

## 課題

実行時間の揺らぎとchannel条件を区別し、再実行可能な形で比較する。主観的音質を根拠なく単一scoreへ変換しない。

## 選択肢

1. CriterionだけでCPU時間を測る。
2. ブラウザUI内だけで計測する。
3. Rust runnerでcodec全体を測り、JSONを出力する。

## 採用した案

選択肢3を採用した。決定論的fixtureを使い、encode/decode中央値、output bytes、playback time、payload bitrate、clean round-trip、5つの決定論的channel成功数を記録する。音楽性は主観scoreではなくprofile設定の`music_weight`と`max_polyphony`をそのまま提示する。

## 採用理由

native codec全体を同じ条件で比較でき、CIや将来の可視化からJSONを再利用できる。新しいbenchmark dependencyも不要である。

## メリット

- 結果schemaとfixtureが再現可能になる。
- MIDI、全音響profile、Secure overheadを一つのreportで比較できる。
- 性能閾値と機能成功条件を分離できる。

## デメリット

- wall-clock値はhardwareと負荷に依存する。
- 決定論的channel modelは実スマートフォン録音を代替しない。
- MIDIの再生時間は現段階のreport対象外である。

## 将来的な見直し条件

実端末recording corpus、Opus fixture、CI performance regression thresholdを導入するときに更新する。

Phase 9完了までに、すべてのchannel reportを`Final Recovery Rate / Raw Symbol Accuracy / Corrected Errors / Payload Bitrate / Decode Time / Playback Duration`の6指標へ統一する。現行schemaにないraw symbolおよびFEC correction telemetryはdecoder境界から追加する。

## Opus fixture追記

Opus往復fixtureは、プリエコーで音長境界が変形するrhythmic profileではなく、単音・固定symbol slotの`FixedFallback`を使う。RustでのOpus往復とPlaywrightでのWeb Audio importの両方を確認対象とする。rhythmic profileの損失圧縮対応は別のdecoder改善として追跡する。
