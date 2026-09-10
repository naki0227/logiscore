# ADR 0007: CRC・SECDED・Interleaving・RepetitionをFEC profile 1とする

- Status: Accepted
- Date: 2026-09-08

## 背景

Phase 4のPCM decoderはclean signalと小さなnoiseを復元できるが、symbol誤りがpacket byteへ到達した場合は訂正できない。音響経路では単発bit errorと連続したburst errorの両方が起こるため、完全復元を最終条件にする保護層が必要である。

## 課題

- 単一bit errorを訂正し、複数bit errorを検出する。
- burst errorを複数codewordへ分散する。
- 訂正符号が見逃す誤りを最終checksumで拒否する。
- 既存のFEC profile `0`とwire互換性を維持する。
- Rust nativeとWASMで同一実装を使い、新規依存を最小化する。

## 選択肢

1. Reed–Solomon crateを導入する。
2. byte repetitionとCRCだけを使用する。
3. Hamming SECDED、bit interleaving、3-copy majority、CRC-32を組み合わせる。

## 採用した案

選択肢3をFEC profile `1`として採用する。

```text
compressed payload
  → CRC-32 append
  → Hamming SECDED (13,8)
  → 8-codeword bit interleave
  → protected stream × 3
  → v2 packet body
```

- SECDEDは各8 data bitsを12-bit Hamming codewordとoverall parity 1 bitへ変換する。
- 8 codewordsをbit列方向に転置し、連続errorを別codewordへ分散する。
- 同じprotected streamを3回格納し、bit単位のmajority voteを行う。
- 復号後のcompressed payloadをIEEE CRC-32で検証してから展開する。
- v2 Binary Headerの`fec_profile=1`で識別し、`payload_length`は全3-copyを含むbody byte長とする。
- Reliable PCM APIだけがprofile `1`を生成し、既存PCM/MIDIはprofile `0`を維持する。

## 採用理由

Phase 5の基準として、単発error訂正、double-bit検出、burst分散、最終完全性検証を小さく監査可能な実装で揃えられる。外部crateを追加しないためWASMサイズ、依存監査、長期保守の増加を抑えられる。

## メリット

- codewordごとのsingle-bit errorを訂正できる。
- double-bit errorをSECDEDで拒否できる。
- 1 copy内の連続burstはmajority voteで復元できる。
- SECDEDで検出できないmulti-bit errorもCRC-32で最終拒否できる。
- profile `0`の既存packetを変更しない。
- correction countをdomain結果として取得できる。

## デメリット

- 約4.875倍に加えてCRC framing分の容量を使い、再生時間が長い。
- Reed–Solomonほど容量効率と多byte訂正能力が高くない。
- Binary Header自体はFEC対象外で、破損時はpacketを拒否する。
- CRC-32は改ざん防止ではない。認証はPhase 8のAEADが担当する。
- profile `1`は固定強度で、環境別の選択はPhase 7まで未対応。

## 将来的な見直し条件

- 実録音benchmarkでoverheadに対する復元率が不十分な場合。
- Reed–Solomon、BCH、LDPC等が明確に優位と確認できた場合。
- header保護または複数FEC strengthが必要になった場合。
- Adaptive Profileが実測error rateからFECを選択する場合。
