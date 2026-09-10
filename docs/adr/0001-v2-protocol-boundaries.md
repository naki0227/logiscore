# ADR 0001: v2プロトコルをv1と並行して導入する

- Status: Accepted
- Date: 2026-09-01

## 背景

v1は圧縮済みバイトをPitch/Velocityへ直接割り当てるDense MIDI方式であり、完全可逆だが、PayloadとTransportが密結合している。v2ではText、File、Projectを共通のPayload層で扱い、Dense、Musical、Acousticへ差し替え可能にする必要がある。

## 課題

- v1互換性を壊さずにv2を開発する。
- MIDIメタイベントに依存しない、Transport共通の短いヘッダーを定義する。
- 将来のMusical/Acoustic実装をPayload処理から分離する。

## 選択肢

1. v1 APIとヘッダーを直接v2へ変更する。
2. v1を維持し、v2のPayload・Binary Header・Transport境界を並行追加する。
3. Core全体を新規crateとして書き直す。

## 採用した案

選択肢2を採用する。既存の `encode` / `decode` とv1 MIDIメタデータは維持し、v2専用APIを別モジュールに追加する。

v2 Binary Headerは7 bytesとする。

| Byte | 内容 |
|---|---|
| 0..1 | Magic 12 bit (`0x4C5`) + Version 4 bit (`2`) |
| 2 | Payload Type 2 bit + Codec Profile 3 bit + FEC Profile 3 bit |
| 3 | Flags 8 bit |
| 4..6 | Payload Length 24 bit、big-endian |

Payload LengthはBinary Header直後に続く圧縮済みPayloadの長さを示す。現行Dense MIDIは `Transport` traitのadapterとして再利用する。

### Source File canonical schema

Source Fileは、Transport投入前に次のbinary表現へ正規化し、全体をzlib圧縮する。整数はbig-endianとする。

| Byte | 内容 |
|---|---|
| 0 | Source File schema version。初期値は`1` |
| 1 | Text encoding。`0`はUTF-8、その他は予約 |
| 2..3 | FilenameのUTF-8 byte長（u16） |
| 4 | ExtensionのUTF-8 byte長（u8） |
| 5..8 | Source contentのUTF-8 byte長（u32） |
| 9.. | Filename、Extension、Source contentの順に連結 |

初期実装では以下を制約とする。

- Filenameは1〜255 bytesとし、`.`、`..`、NUL、`/`、`\`を拒否する。
- Extensionは0〜32 bytesとし、ASCII英数字と`.`、`-`、`_`、`+`のみ許可する。`.rs`形式に加えて`Dockerfile`などの言語識別子を保持できる。
- Source contentはUTF-8、最大8 MiBとする。
- 宣言長と実データ長が一致しないpacket、および末尾余剰データを拒否する。
- v2の展開結果は8 MiB + metadata上限で打ち切り、decompression bombを拒否する。
- Unicode normalizationは行わず入力bytesを保持する。ExtensionはFilenameから推測せず、独立したmetadataとして保持する。

FilenameとExtensionを分離することで、拡張子のない論理名やUI上の出力名を損なわず、言語・音色選択用のExtensionを明示的に扱える。UTF-8以外のencoding IDは将来拡張用に予約する。

## 採用理由

- v1利用者と既存MIDIの復号互換性を維持できる。
- Payload、圧縮、packet、transportの責務が分離され、各層を単体テストできる。
- 新しい外部依存を追加せずに開始できる。
- Musical/Acoustic transportを同じtraitへ段階的に追加できる。

## メリット

- 移行を段階化でき、回帰リスクが低い。
- 7-byte headerにより短いTextでもオーバーヘッドを抑えられる。
- 不正magic、version、length、payload typeをTransportの外側で一貫して拒否できる。

## デメリット

- 移行期間中はv1/v2のAPIとヘッダーが併存する。
- 24-bit lengthにより単一packetは16 MiB未満に制限される。
- v2 Denseは当面v1のVelocity依存方式を内部利用するため、録音耐性は持たない。
- Source Fileは当面UTF-8かつ8 MiB以下に限定され、権限・更新日時などのfilesystem metadataは保持しない。

## 将来的な見直し条件

- 16 MiB以上の単一Payloadをchunk化せず扱う必要が生じた場合。
- FECや暗号化によりheader extensionが必要になった場合。
- Transportごとに同期情報を共通headerへ持たせる必要が判明した場合。
- v1互換APIの利用がなくなり、移行完了を判断できた場合。
- UTF-16やbinary fileなどUTF-8以外のSource Fileを扱う要件が確定した場合。
