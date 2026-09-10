# ADR 0002: v2 Projectを長さ付きcanonical archiveで表現する

- Status: Accepted
- Date: 2026-09-04

## 背景

Project payloadは複数のUTF-8 source fileと相対pathを、入力順やブラウザ実装に依存せず同一bytesへ正規化する必要がある。外部archive依存の追加、path traversal、重複path、過大展開も避ける必要がある。

## 課題

- 同一Projectから決定的なpayloadを生成する。
- 不正pathとdecompression bombを境界で拒否する。
- v1 Project MIDIのimport互換性を維持する。
- 将来のTransportからfilesystem表現を分離する。

## 選択肢

1. ZIPまたはtarをcanonical archiveとして採用する。
2. JSONへBase64 sourceを格納する。
3. version付きheaderと長さ付きentryからなる専用binary schemaを採用する。

## 採用した案

選択肢3を採用する。整数はbig-endian、文字列はUTF-8とし、entryはpathのUTF-8 bytes順に整列する。

Project header:

| Byte | 内容 |
|---|---|
| 0 | Schema version。初期値は`1` |
| 1 | Text encoding。`0`はUTF-8 |
| 2..3 | File count（u16） |
| 4..7 | 全sourceのbyte長（u32） |

各entry:

| Byte | 内容 |
|---|---|
| 0..1 | Relative pathのbyte長（u16） |
| 2 | Extensionのbyte長（u8） |
| 3..6 | Sourceのbyte長（u32） |
| 7.. | Path、Extension、Sourceの順に連結 |

制約は次のとおりとする。

- 1〜2,048 files。
- pathは相対path、最大1,024 bytes、各segment最大255 bytes。
- 空segment、`.`、`..`、先頭・末尾`/`、backslash、colon、NUL、control characterを拒否する。
- pathの重複とcanonical順序でないpacketを拒否する。
- extensionは最大32 bytesで、Source File schemaと同じ文字制約を使う。
- sourceは1 fileあたり8 MiB、合計32 MiBまでとする。
- 宣言長不一致、切断、末尾余剰data、未対応schema/encodingを拒否する。
- regular fileのみ表現し、symlink、directory entry、権限、時刻、所有者は格納しない。
- Dense packetの圧縮後payloadはv2 headerの24-bit上限にも従う。

## 採用理由

- ZIP/tarのmetadata差異を正規化する追加依存が不要になる。
- JSON/Base64より小さく、長さ検証を段階的に行える。
- archive schemaをTransportおよびWeb File APIから独立して単体テストできる。
- path bytesによる整列で入力順に依存しない。

## メリット

- 決定的encodeと厳格なdecodeが可能になる。
- path traversalや重複上書きをarchive境界で拒否できる。
- v2 Text/Source Fileと同じ圧縮・packet・Transport経路を再利用できる。

## デメリット

- 一般的なarchive toolでは直接開けない。
- binary fileとfilesystem metadataは保持できない。
- 32 MiB以下でも圧縮後に24-bit上限を超えるProjectはDense transportでencodeできない。
- schema拡張時はversioningが必要になる。

## 将来的な見直し条件

- binary assetまたはsymlinkを保持する要件が確定した場合。
- 2,048 filesまたは32 MiBを超えるProjectをchunk化する場合。
- filesystem metadataの再現が必要になった場合。
- v2 headerのpayload lengthまたはstreaming方式を変更する場合。
