# ADR 0004: Musical MIDIを4声chordによるcodec profile 1として実装する

- Status: Accepted
- Date: 2026-09-08

## 背景

Musical domain codecで4 bit symbolと4声voicingの可逆変換が成立した。これを標準MIDIファイルへ格納し、v2 packetおよびWeb UIからDense profileと選択可能にする必要がある。

## 課題

- MIDI内で4声voicingとevent境界を決定的に表現する。
- Velocityへdataを格納せず、音程だけからsymbolを復元する。
- Dense MIDIとの誤認を防ぐ。
- 過大な入力によるmemory・生成時間の増大を制限する。

## 選択肢

1. 既存Dense MIDIのPitch/Velocity割当を装飾して再利用する。
2. 1 symbolを4声同時発音の1 chordとして新規Transportにする。
3. 外部MIDI生成libraryを追加してmulti-track形式を構築する。

## 採用した案

選択肢2を採用する。

- v2 Binary Headerのcodec profile `1`をMusical MIDIとする。
- 1 byteをhigh/low nibbleへ分割し、2個の4 bit symbolとして送る。
- 1 symbolはBass、Harmony 2、Harmony 1、Melodyの4 channel同時発音とする。
- PPQ 480、note duration 360 ticks、rest 120 ticks、tempo 120 BPMとする。
- Velocityは全Note Onで80固定とし、data channelとして使用しない。
- Track Name `Logiscore v2 Musical`を識別markerとする。
- MIDIはType 0 single trackとし、decoderは4 channelが揃ったeventだけを受理する。
- 圧縮済みv2 packetは64 KiBまで、最大symbol数は131,072とする。
- Text、Source File、Projectの全payloadで同じTransportを利用する。
- import decoderはDenseを試した後にMusicalを試し、既存v1 fallbackも維持する。

## 採用理由

- `MusicalSymbolCodec`をそのまま利用でき、domainへMIDI知識を持ち込まない。
- note pitchだけで完全可逆となり、Velocity依存を除去できる。
- 手書きの小さなType 0 writerと既存midly parserで外部依存を増やさない。
- codec profileによりpacket解釈を明示できる。

## メリット

- MIDI file内でText、Source File、Projectが100% round-tripする。
- 4声、和声、voice leadingを実際の再生データへ反映できる。
- Dense profileを残したままMusic modeを追加できる。
- Playwrightで利用者操作を継続的に検証できる。

## デメリット

- 4 bit/eventかつ2 events/byteのためDenseよりMIDIが大きく再生時間も長い。
- 固定tempo、固定duration、固定velocityで表現力が限定される。
- profile上限を超えるProjectはMusic modeでencodeできない。
- MIDI fileの可逆性であり、録音音声からの復号はまだ保証しない。

## 将来的な見直し条件

- Phase 3でrhythmや複数symbol channelを導入する場合。
- candidate countを動的に変更する場合。
- profile headerへtonality、tempo、instrument情報を追加する場合。
- Audio Decoderで固定duration・channel構成が不利と判明した場合。
