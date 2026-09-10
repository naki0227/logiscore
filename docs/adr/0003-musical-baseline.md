# ADR 0003: Musical Baselineを決定的な16候補・4声モデルで構築する

- Status: Accepted
- Date: 2026-09-08

## 背景

v2 Phase 2では、データを音程へ直接割り当てるDense方式とは別に、現在の調性・和声・直前のvoicingから音楽的に妥当な候補集合を作り、4 bit symbolで候補を選ぶ基盤が必要になる。encoderとdecoderが候補集合を完全に再構築できなければ可逆にならない。

## 課題

- 候補生成を決定的かつTransport非依存にする。
- 4声の音域とvoice crossing禁止を保証する。
- voice leadingとconsonanceを定量評価する。
- 16候補から4 bit symbolを完全可逆に変換する。

## 選択肢

1. MIDI noteへ固定テーブルで直接割り当てる。
2. 確率的・生成AIベースでvoicingを生成する。
3. Tonal Context、Chord Planner、制約付き候補列挙、決定的score順を組み合わせる。

## 採用した案

選択肢3を採用する。初期profileは次の固定仕様とする。

- Tonal Contextはroot pitch classとMajor、Natural Minor、Dorian、Lydianを持つ。
- Chord progressionは4 event周期の`I → V → vi → IV`とし、各modeのscale degreeからtriadを作る。
- 声部は低音からBass、Harmony 2、Harmony 1、Melodyの4声とする。
- 各声部のrange内にあるchord toneを列挙し、strict ascendingを満たすvoicingだけを候補にする。
- 前voicingとの共通音・stepwise motion・小さい総移動量を加点し、大跳躍、過密な低音、dissonant intervalを減点する。
- score降順、同点時はMIDI note列の辞書順で整列し、先頭16件をcanonical candidate setとする。
- symbol `0..15`をcandidate indexとして選択し、decodeは観測voicingのindexを返す。

## 採用理由

- encoderとdecoderが同じ状態から同じ候補順を再構築できる。
- 音楽規則をMIDI parserやPayload schemaから分離して単体テストできる。
- 新しい依存を追加せず、将来score weightやprofileをversion管理できる。

## メリット

- 4 bit/eventの可逆なbaselineになる。
- 4声域、voice crossing、和声音という最低限の音楽制約を構造で保証できる。
- Dense、Musical MIDI、Acousticの各Transportから再利用できる。

## デメリット

- 固定進行であり、長い曲では反復感が強い。
- 全声部をchord toneに限定するため旋律の自由度が低い。
- score weight変更はcandidate順を変えるためcodec profile versionの変更が必要になる。
- Phase 2段階では録音音声からの復号耐性を保証しない。

## 将来的な見直し条件

- Rhythm Encodingやmultiple symbol channelを導入する場合。
- non-chord tone、転調、複数進行を候補へ含める場合。
- score weightを変更する場合。
- Acoustic decoderの識別率からvoice rangeやinterval制約を変更する場合。
