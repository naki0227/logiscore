# ADR 0013: v2 packetの外側にcheckpoint loop envelopeを置く

- Status: Accepted
- Date: 2026-09-12

## 背景

現行音響decoderは録音の最初のonsetをpreambleとして扱い、1つの連続packetを先頭から最後まで必要とする。このため途中から録音した場合、再生がloopしていてもpayloadを復元できない。

## 課題

- 録音開始位置に依存せず、次に現れる同期点からchunkを取得する。
- `Chunk 2 / Chunk 3 / Chunk 0 / Chunk 1`のような順不同観測を元の順序へ戻す。
- 複数loopで得た重複観測を統合し、単発の誤りを改善する。
- 壊れたchunk、別transferの混入、欠落、重複、過大入力を安全に扱う。
- 既存v2 packet、payload type、Secure envelopeとの互換性を維持する。

## 選択肢

1. 既存v2 Binary Headerへchunk fieldを追加する。
2. 音響decoderだけで固定時間ごとに切り、protocol metadataを持たない。
3. 完成済みv2 packetの外側をversioned checkpoint envelopeで分割する。

## 採用した案

選択肢3を採用する。

```text
canonical payload
  → existing v2 packet / Secure packet
  → checkpoint split
  → [Sync][Checkpoint Envelope][Chunk CRC]
  → loop
```

checkpoint envelope version 1は次を持つ。

```text
Magic                4 bytes
Version              1 byte
Transfer ID          4 bytes (full packet CRC-32)
Chunk Index          2 bytes
Chunk Count          2 bytes
Total Packet Length  4 bytes
Chunk Length         2 bytes
Chunk Payload        0..768 bytes
Chunk CRC-32         4 bytes
```

- 既定chunk payloadは512 bytes、上限は768 bytesとする。
- 空payloadも1つの空chunkとして表す。
- chunkはindex順でなくてもよい。全indexが揃った時点で再構成する。
- 同一indexの重複観測は完全一致数の多数決を行い、同数なら曖昧として拒否する。
- 各chunk CRCに加えて、再構成後の全packet CRCをTransfer IDと照合する。
- 別Transfer ID、chunk count、total lengthが混ざった入力は拒否する。
- 音響層は各envelope前の同期点を探索する。symbol confidenceを使うsoft combineは、hard observation voteの次段階として同じenvelopeへ接続する。

## 採用理由

既存packetの内側には圧縮、FEC、Secure authenticationがあり、そこへchunk責務を混ぜると全transportへ破壊的影響が出る。外側で分割すればText、Source File、Project、Secureを区別せず再利用でき、音響同期方式もversioned boundaryで改善できる。

## メリット

- 既存v2 wire formatを変更しない。
- 任意開始位置とloop順序の責務をpayload codecから分離できる。
- chunk単位でCRC、重複排除、多数決、欠落判定ができる。
- 全payload CRCにより誤った多数決や混在を最終拒否できる。
- 新規dependencyを追加しない。

## デメリット

- 各chunkに23 bytesの固定overheadが付く。
- CRC-32のTransfer IDは暗号学的識別子ではない。
- hard voteはCRCを通った観測しか統合できず、symbol confidenceによるsoft combineより情報量が少ない。
- checkpointごとの同期音によりbitrateと音楽性が下がる。

## セキュリティとエラー方針

長さ、chunk数、index、version、CRCを外部入力として検証する。CRCは偶発誤り検出だけに使い、改ざん耐性は既存Secure envelopeのAEADが担う。混在、欠落、tie、CRC不一致の詳細は内部診断に使える型付きerrorへ変換し、Web UIには既存の一般化した録音失敗messageを返す。

## 将来的な見直し条件

- 実録音で512-byte chunkが長すぎ、短いcheckpointが有意に復元率を上げる場合。
- Transfer ID衝突が運用上無視できなくなり、より長いdigestが必要な場合。
- soft confidence combineがhard voteより有意に改善する場合。
- 音楽的な同期motifが機械的Mini Syncと同等の検出率を達成する場合。
