# ADR 0009: Environment・calibration・priorityからAcoustic Profileを決定する

- Status: Accepted
- Date: 2026-09-09

## 背景

Phase 6のReliable modeは単一の8 kHz PCM/FEC設定を使う。実際の音響経路ではquiet room、conversation、noise、online call、long distanceで適切なsymbol durationと冗長度が異なる。

## 課題

- user指定とcalibration測定の両方から決定的にprofileを選ぶ。
- 選択根拠のconfidenceとfallback順をUI・decoderへ渡す。
- payload sizeとMusic/Reliability priorityを所要時間へ反映する。
- 既存Balanced WAVをdecode互換の先頭候補として維持する。

## 選択肢

1. environment名をUI表示だけに使い、codecは変えない。
2. 機械学習modelでprofileを推定する。
3. versioned profile tableと決定的なthreshold selectorをRust domainに置く。

## 採用した案

選択肢3を採用する。

- `Quiet`、`Balanced`、`Conversation`、`Noisy`、`Online`、`LongDistance`、`FixedFallback`を固定IDで定義する。
- profileはtiming倍率、polyphony、FEC、interleave、repetition、music weightを保持する。ConversationはFEC 2（depth 16 / 3-copy）、Noisy・Online・Long DistanceはFEC 3（depth 16 / 5-copy）を使用する。
- explicit environmentはconfidence 100とする。
- Autoはnoise floor、SNR、clipping ratio、reverberationから環境を判定する。calibrationがなければBalanced/confidence 50とする。
- reliability priorityが高ければQuietからBalancedへ、速度優先かつ大payloadならBalancedからQuietへ調整する。
- decoderはcalibration結果を先頭候補にし、失敗時に全profileとFixedFallbackを重複なしの決定順で試す。
- estimated durationはpacket event数とprofile timing倍率から計算する。
- acoustic profile IDはv2 Binary Headerのflagsに格納し、復号後に期待profileと一致することを検証する。flags 0の旧Balanced Reliable WAVも最後に再試行して後方互換を保つ。
- FixedFallbackはcodec profile 3の単声16-tone codecとして実装する。

## 採用理由

決定的なtableとthresholdはsender/receiver、native/WASM間で再現でき、テストと説明が容易である。ML modelや外部serviceが不要で、calibration dataを収集した後もthresholdをversion管理できる。

## メリット

- environmentごとのtrade-offを明示できる。
- confidenceとfallbackをuserへ説明できる。
- profile選択とsignal codecを分離して単体テストできる。
- playback durationをencode前に表示できる。

## デメリット

- 初期thresholdは人工channel testに基づき、実機benchmarkで調整が必要である。
- 複数profile decodeはCPU時間を増やす。
- profile IDとtiming規則は新しいwire互換性になる。
- 初期calibrationはnoise floor、SNR、clipping、末尾減衰の近似であり、speech/codec artifact専用classifierはPhase 9実測後の見直し対象である。

## 将来的な見直し条件

- Phase 9の実測でenvironment分類精度が不足する場合。
- codec artifactやspeech interferenceの特徴量を追加する場合。
- profile tableをremote/configurableにする必要が生じた場合。
- ML selectionが決定的thresholdを明確に上回る場合。
