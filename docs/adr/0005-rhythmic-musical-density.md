# ADR 0005: Rhythmと動的候補数でMusical bitrateを高密度化する

- Status: Accepted
- Date: 2026-09-08

## 背景

codec profile `1`は4声Pitch候補だけで4 bit/eventを送る。Phase 3では音楽性を保ちながらevent数とMIDI sizeを減らすため、Rhythm Encoding、Multiple Symbol Channels、Dynamic Candidate Countが必要になる。既存profileの候補順やtimingを変えると復号互換性を壊す。

## 課題

- Pitch以外の音楽要素へdataを分散する。
- 文脈に応じて候補数を変えてもdecoderが同じ容量を再現できるようにする。
- bit paddingと元packet長を安全に扱う。
- profile `1`との後方互換性を維持する。

## 選択肢

1. profile `1`のdurationを直接変更する。
2. Velocityへ追加bitを格納する。
3. 新profile `2`でPitch候補とRhythmを独立symbol channelとして使う。

## 採用した案

選択肢3を採用する。

- 強拍相当の偶数eventは8 Pitch候補（3 bit）、奇数eventは32候補（5 bit）とする。
- Rhythmは240、360、480、720 ticksの4候補（2 bit）とする。
- event容量は交互に5 bit、7 bitとなり、平均6 bit/eventとする。
- 入力をMSB firstのbitstreamとして読み、最終eventの不足bitは0 paddingする。
- Track Text metadata `L:<packet bytes>`で元packet長を保持し、decoderは余剰paddingが0であることを検証する。
- codec profile `2`とTrack Name `Logiscore v2 Rhythmic`で識別する。
- Note On velocityは80固定とし、Rhythmは4声すべてのNote Off tickから復元する。
- WebのMusic modeは新規encodeでprofile `2`を使い、importはprofile `2`、profile `1`、Dense、v1の順に互換decodeする。

## 採用理由

- 既存profile `1`のwire互換性を保持できる。
- Pitchとdurationという独立した音楽要素へdataを分散できる。
- event位置だけからcandidate countを決定でき、追加状態を共有しなくてよい。
- 256 bytes比較でprofile `1`より生成MIDIが小さくなることをテストできる。

## メリット

- 4 bit/eventから平均6 bit/eventへ改善する。
- duration変化が加わり、固定リズムより音楽的な変化が増える。
- Dynamic Candidate Countをdomain APIとして将来のprofileから再利用できる。
- 全payloadでMIDI file round-tripを維持する。

## デメリット

- MIDI metadataにpacket byte長を保持する。
- 5/7 bitの可変容量により実装と検証が複雑になる。
- 固定4種類のdurationであり、syncopationやrestはまだ扱わない。
- 録音後のduration揺らぎを許容するdecoderはPhase 4以降となる。

## 将来的な見直し条件

- Audio Decoderのtiming誤差からduration候補間隔を変更する場合。
- rest、accent、articulationをsymbol channelへ追加する場合。
- 32を超える候補または5声以上を使う場合。
- MIDI metadataに依存しないlength framingを導入する場合。
