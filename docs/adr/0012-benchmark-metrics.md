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

### 6共通指標の定義

- Final Recovery Rate: 全channel caseに対するPayload完全一致数の割合。
- Raw Symbol Accuracy: 音響decoderがFEC前に復元したpacket bitとclean packet bitの一致率。長さ不一致またはraw packetを返せないcaseは0%とし、失敗caseを除外して過大評価しない。
- Corrected Errors: raw packetとclean packetの異なるbitのうち、最終Payloadが完全一致したcaseのbit数。RepetitionとHammingを含むend-to-endの訂正効果として数える。
- Payload Bitrate: canonical payload bit数をPlayback Durationで割った値。
- Decode Time: 同一入力に対するdecode wall-clockの中央値。
- Playback Duration: 出力音声sample数とsample rateから求める。

MIDIはPure Digital baselineとしてclean round-trip成功時にFinal RecoveryとRaw Accuracyを100%、Corrected Errorsを0とする。report schema v2で全6指標を必須fieldにする。

## Opus fixture追記

Opus往復fixtureは、プリエコーで音長境界が変形するrhythmic profileではなく、単音・固定symbol slotの`FixedFallback`を使う。RustでのOpus往復とPlaywrightでのWeb Audio importの両方を確認対象とする。rhythmic profileの損失圧縮対応は別のdecoder改善として追跡する。

## 実録音telemetry境界

実録音では期待するTextからprofile別のclean packetを再構成し、音響decoderがFEC前に返したraw packetとbit単位で比較する。ブラウザ側はWeb AudioでM4A/WAVをPCMへ変換し、Rust/WASM境界からFinal Recovery Rate、Raw Symbol Accuracy、Corrected Errorsを取得する。Payload Bitrate、Decode Time、Playback Durationを加えた6指標をPlaywright artifactへ出力する。

実録音が復元できない場合も0%として記録し、成功caseだけを母数にして性能を過大評価しない。decode wall-clockだけは実行環境依存のため、corpusのverified値との厳密一致対象にしない。

最初の1m iPhone録音では、送信WAV比で平均levelが約17 dB低下し、一部symbolのGoertzel energyが従来の絶対floorを下回った。一方で同期周波数の相対優位は維持され、raw packetの誤りは3 bitだけだった。このためFixedFallbackは同期の相対判定を維持しつつsymbol energy floorを下げ、FECへraw packetを渡す方針とした。無音・非有限値・不正framingの拒否は維持する。
