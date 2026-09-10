# Logiscore 改善設計書
## Musical Codec / Acoustic Transport / Secure & Adaptive Design

## 1. この文書の目的

Logiscore は、ソースコードやテキストなどの情報を「音楽として表現し、必要に応じて元の情報へ戻せる」システムである。

現行実装では、ソースコードを圧縮したバイト列を MIDI のノート情報へ直接割り当てることで、可逆な Code to Music を実現している。一方で、情報密度を優先した結果、同時発音数が増えやすく、音楽性・録音耐性・音響通信としての堅牢性には改善余地がある。

本設計では、Logiscore を単なる「ソースコードをMIDI化するツール」から、次のようなシステムへ発展させる。

> **情報を、音楽として聴ける形に可逆変換し、必要なら実際の音声・録音・通信路を通して復元できる Musical Codec / Acoustic Transport**

特に以下を重視する。

- 可逆性
- 音楽性
- 情報密度
- 録音耐性
- ノイズ耐性
- 環境適応
- シンプルなUX
- セキュアな共有
- 拡張可能なプロトコル設計

---

# 2. 設計思想

## 2.1 「音楽らしさ」と「情報量」を分離して考える

現行方式では、圧縮後のバイト値がそのまま Pitch / Velocity に影響する。

しかし、圧縮後データは音楽的意味を持たない。

そのため、

```text
compressed bytes
      ↓
pitch / velocity
      ↓
music
```

という直接変換では、可逆性は高い一方で、旋律・和声・声部進行が不自然になりやすい。

今後は、

```text
data
 ↓
musical context
 ↓
musically valid candidates
 ↓
data selects one candidate
```

という構造にする。

つまり、

> **音楽理論が「この音を鳴らす」と決めるのではなく、「この中ならどれを選んでも音楽として成立する」という符号表を作り、データがその候補を選ぶ。**

これを Logiscore の中心的な設計思想とする。

---

## 2.2 可逆性は最優先制約

音楽性を上げるために生成AIやランダムな作曲アルゴリズムを使用すると、復号側で同じ状態を再構築できなくなる可能性がある。

そのため Musical Codec は原則として決定論的にする。

同じ入力状態からは、送信側・受信側が必ず同じ Candidate Set を再構築できなければならない。

---

## 2.3 「どの環境でも完璧」ではなく「環境に合わせる」

音響通信では、

- 部屋の反響
- スピーカー特性
- マイク特性
- 距離
- 人の声
- 音楽
- AAC / MP3 / Opus
- AGC
- Noise Suppression
- Echo Cancellation

などによって信号が変形する。

単一プロファイルで全環境を最適化することは現実的ではない。

そのため、

> **自動である程度うまく動き、環境を指定するとさらに精度を上げられる**

設計を採用する。

---

## 2.4 高速通信の代替を目指さない

Logiscore は Wi-Fi、Git、HTTP、QRコードなどの代替を目的としない。

大規模データでは再生時間が長くなってよい。

むしろ、

> 「技術的には送れるが、音響帯域なので数十分・数時間かかる」

という制約もシステムの性質として明示する。

この制約自体を隠さず、情報量・再生時間・音楽性・耐ノイズ性のトレードオフを可視化する。

---

## 2.5 普通のユーザーには複雑さを見せない

内部では、

- FEC
- symbol duration
- interleave depth
- polyphony
- frequency set
- codec profile
- guard interval

など多くのパラメータを持つ。

しかし通常のUIでは、

```text
Mode
● Auto
○ Music
○ Reliable
○ Fast

Environment
● Auto
○ Quiet room
○ Conversation nearby
○ Noisy room
○ Online call
○ Long distance
```

程度に留める。

詳細設定は Advanced Settings に分離する。

---

# 3. 対象Payload

Logiscore はソースコード専用に限定しない。

将来的には以下を同じ基盤で扱う。

```text
Payload
├── Text
├── Source File
└── Project
```

## Text

最小の通信単位。

例：

```text
Hello from Logiscore.
```

```text
明日13時集合
```

短いメッセージではファイル名・言語情報・ディレクトリ構造が不要なため、最も小さいヘッダーで送れる。

Acoustic Mode の最初のデモにも向いている。

## Source File

1ファイル単位。

必要に応じて、

- filename
- language / extension
- encoding
- content

を保持する。

## Project

複数ファイル・ディレクトリを含む。

必要に応じて archive 化してから圧縮し、単一 Payload として扱う。

---

# 4. 現行方式の主な課題

## 4.1 Byte と音が直接結びついている

現在は概念的に、

```text
upper 4 bit → pitch
lower 4 bit → velocity
```

のような形で情報を持たせている。

MIDI内部では扱いやすいが、圧縮済みデータに音楽的意味がないため、自然な旋律にはなりにくい。

---

## 4.2 再生時間短縮による同時発音の増大

大規模プロジェクトほど短時間に収めるため、複数バイトを同一 tick に配置する。

結果として、

- polyphonyの増大
- 倍音の重なり
- masking
- roughness
- 録音時の識別困難

が発生する。

Dense Mode としては有効だが、Music / Acoustic Mode には向かない。

---

## 4.3 Velocity は Acoustic Transport に向かない

Velocity は、

- 再生音量
- マイク感度
- 距離
- AGC
- 圧縮
- ノイズ抑制

などで容易に変化する。

そのため Acoustic Mode では Velocity を主要な情報チャネルとして使用しない。

---

## 4.4 ヘッダーが相対的に大きい

現行方式では可読性を重視したメタデータを持っているため、短いPayloadほどヘッダー比率が大きくなる。

特に Text や数行のコードでは本編よりヘッダーの存在感が大きくなりやすい。

今後は、

- MIDI Transport
- Acoustic Transport

でヘッダー設計を分離する。

---

# 5. 全体アーキテクチャ

```text
Source / Text / Project
        ↓
Canonicalizer
        ↓
Archive (optional)
        ↓
Compression
        ↓
Plain Bitstream
        ↓
Encryption (optional)
        ↓
FEC / Interleaving
        ↓
Transport Encoder
      /       |        \
     /        |         \
Dense      Musical    Acoustic
MIDI       Codec      Codec
              ↓          ↓
           Musical IR   Audio
              ↓          ↓
             MIDI      Speaker
                         ↓
                        Air
                         ↓
                    Microphone
                         ↓
                   Audio Decoder
                         ↓
                    Bitstream
                         ↓
                    Decrypt
                         ↓
                    Decompress
                         ↓
                 Original Payload
```

---

# 6. Transport の分離

内部ではPayloadとTransportを明確に分ける。

```text
Payload Layer
├── Text
├── File
└── Project

Transport Layer
├── Dense MIDI
├── Musical MIDI
├── Acoustic
└── Fixed Fallback
```

これにより、同じPayloadを複数の表現方法へ送れる。

---

# 7. Dense Mode

現行方式に近い。

## 目的

- 最短時間
- 高情報密度
- MIDI内部での完全可逆

## 特徴

- 高polyphonyを許容
- 音楽性は低〜中
- 録音復元は保証しない
- Velocity利用可
- 大規模プロジェクト向き

現行ロジックは廃止せず、このモードとして残す。

---

# 8. Music Mode

## 目的

- 音楽として自然
- 完全可逆
- MIDIとして扱いやすい

## 基本構造

```text
Current Musical State
        ↓
Candidate Generator
        ↓
Valid Candidate Set
        ↓
Data selects index
        ↓
Musical Event
```

---

# 9. Candidate-Based Encoding

例として4bitを1イベントに持たせる場合、

```text
candidate[0]
candidate[1]
...
candidate[15]
```

の16候補を作る。

入力：

```text
1011
```

なら、

```text
1011 = 11
→ candidate[11]
```

を選択する。

復号側も同じ状態を再構築し、

```text
observed candidate
→ index 11
→ 1011
```

と戻す。

---

# 10. Musical Context

Candidate Generator は以下を参照する。

## Tonality

- Major
- Minor
- Dorian
- Lydian
- Pentatonic
- その他

## Harmony

例：

```text
I → V → vi → IV
```

将来的には複数の進行を持つ。

## Beat Position

- strong beat
- weak beat
- pickup
- syncopation

## Previous Notes

前の音との距離を評価する。

## Voice Range

各声部の音域を制約する。

---

# 11. Voice Architecture

初期実装では最大4声を基本とする。

```text
Melody
Harmony 1
Harmony 2
Bass
```

目的：

- 同時発音数の抑制
- voice crossing防止
- 音域整理
- acoustic separability向上

---

# 12. Voice Leading

候補の評価例：

```text
score(note) =
    chordToneBonus
  + commonToneBonus
  + stepwiseMotionBonus
  + contraryMotionBonus
  - leapPenalty
  - crossingPenalty
  - dissonancePenalty
  - rangePenalty
```

ルール例：

- 同音維持を高評価
- 半音・全音・短3度程度の移動を優先
- 大跳躍を減点
- 強拍はコードトーンを優先
- voice crossing禁止
- 低音域では密集を避ける
- 不協和音は文脈上許容される場所に限定

---

# 13. Rhythm Encoding

Pitch だけではなく Rhythm にもデータを分散する。

例：

```text
00 → eighth
01 → quarter
10 → dotted quarter
11 → half
```

その他、

- rest
- syncopation
- articulation
- accent

なども候補にできる。

これにより1イベントあたりの情報量を増やし、polyphonyを減らせる。

---

# 14. Acoustic Mode

## 目的

実際の音として再生し、録音した音声から元データを復元する。

対象：

- WAV
- M4A
- MP3
- smartphone recording
- speaker → microphone
- WebRTC / online call

## 基本方針

- Velocity非依存
- 周波数・時間構造を中心に判定
- 少ないpolyphony
- FEC
- Interleaving
- Preamble
- Timing Recovery
- CRC / AEAD authentication

---

# 15. Acoustic Packet

Acoustic Mode では必要情報を音側に持たせる。

ユーザーが別途共有する情報は、Secure Mode で使用する Password のみとする。

概念構造：

```text
Preamble
Protocol Version
Flags
Payload Type
Codec Profile
Payload Length
Salt
Nonce
FEC Profile
Encrypted / Plain Payload
Authentication Tag / Checksum
```

---

# 16. ヘッダー最適化

Acoustic Transport では文字列メタデータを使わず、固定長または短いBinary Headerにする。

例：

```text
Magic         12 bit
Version        4 bit
Payload Type   2 bit
Codec Profile  3 bit
FEC Profile    3 bit
Flags          8 bit
Length        24 bit
```

必要な値だけを最小限持つ。

一方、MIDI Transportではデバッグしやすさを重視し、人間が読めるメタデータを残してよい。

---

# 17. ヘッダーの設計原則

短いText Payloadではヘッダー比率が大きくなるため、以下を優先する。

1. 固定値は送らない
2. Protocol Version から導出できる値は省略する
3. Profile ID から導出できる設定値は省略する
4. 必要な値だけbit packingする
5. Payload Typeごとに最小ヘッダーを定義する
6. optional field は flags で存在有無を示す

---

# 18. Fixed Fallback Codec

Adaptive / Musical Codec が使えない場合に備え、単純で確実な固定変換を残す。

例：

```text
symbol 0  → C4
symbol 1  → D4
symbol 2  → E4
...
```

目的は音楽性ではなく、

> **復号可能性を最優先する最後の安全策**

とする。

## 使用例

- Candidate reconstruction失敗
- Musical Profile非対応
- Decoder version差
- ノイズ環境で複雑な判定が不安定
- ユーザーが Reliable を最優先した場合

---

# 19. フォールバック戦略

```text
Auto Musical Codec
        ↓
decode success?
   ┌────┴────┐
  yes        no
   ↓          ↓
return    Robust Profile
              ↓
        decode success?
          ┌───┴───┐
         yes      no
          ↓        ↓
       return   Fixed Codec
```

必要に応じて複数profileを再試行する。

---

# 20. Environment Profiles

内部構造例：

```text
AcousticProfile {
    symbol_duration
    max_polyphony
    frequency_set
    fec_strength
    interleave_depth
    repetition
    bitrate
    music_weight
}
```

## Auto

安全寄りの一般設定。

## Quiet Room

- 高bitrate
- 弱めFEC
- 短いsymbol
- 音楽性を高くできる

## Conversation Nearby

- 人声との衝突を考慮
- 長めsymbol
- 強めFEC
- deep interleaving
- polyphony削減

## Noisy Room

- さらに強いFEC
- repetition増加
- bitrate低下

## Long Distance

- symbol duration増加
- robustness優先
- 高音圧差に耐える設定

## Online Call

- Opus
- AGC
- Noise Suppression
- Echo Cancellation
- Voice Activity Detection

の影響を前提にする。

---

# 21. Auto Mode

送信側はユーザー指定がなければ、Payload Sizeと用途からprofileを自動選択する。

受信側は録音から、

- noise floor
- approximate SNR
- clipping
- speech interference
- codec artifacts
- frequency response
- reverberation

を解析し、decoder profileを自動選択する。

自動判定に失敗した場合は、複数profileを試行する。

---

# 22. UX

通常画面：

```text
Payload
[ Text / File / Project ]

Mode
● Auto
○ Music
○ Reliable
○ Fast

Environment
● Auto
○ Quiet
○ Conversation
○ Noisy
○ Online
○ Long Distance

Priority
Music ─────●──── Reliability

Estimated Duration
42 sec
```

Advanced Settings：

```text
Codec
FEC Strength
Symbol Duration
Interleave Depth
Max Voices
Frequency Set
Fallback Codec
```

---

# 23. Text Transmission

Textは重要なユースケースとする。

理由：

- ヘッダーが小さい
- 実装確認が簡単
- デモが分かりやすい
- 通信路として成立していることを示しやすい

例：

```text
Input:
Hello from Logiscore.

↓ PLAY

♪ ♫ ♪ ♬

↓ smartphone recording
↓ upload M4A
↓ decode

Output:
Hello from Logiscore.
```

---

# 24. Secure Mode

一部の人だけが復元できるようにする場合、事前共有したPasswordを利用する。

ユーザー間で共有するテキスト情報はPasswordだけとする。

```text
Password
   ↓
Argon2id + random salt
   ↓
Encryption Key
   ↓
ChaCha20-Poly1305
   ↓
Ciphertext
   ↓
FEC
   ↓
Musical / Acoustic Codec
```

Salt、Nonce、Version等は音声側に含める。

---

# 25. Secure Mode の設計思想

Passwordそのものを直接Encryption Keyとして使わない。

人間のパスワードからは Argon2id 等のPassword KDFを使用して鍵を導出する。

暗号方式には AEAD を使用する。

候補：

- ChaCha20-Poly1305
- AES-GCM

復号時にAuthentication Tagが通らなければ、元情報は表示しない。

```text
Correct password
→ authentication success
→ restore

Wrong password
→ authentication failure
→ reject
```

---

# 26. Error Correction

音響通信ではRaw symbol errorを0にすることを目標としない。

成功条件は、

> **最終的にPayloadが100%復元できること**

とする。

導入候補：

- Reed-Solomon
- CRC
- Interleaving
- Symbol repetition
- Preamble
- Guard interval
- Timing synchronization

---

# 27. 人の声への耐性

人声がある環境でも一定の復元性を狙う。

ただし「人が喋っていても必ず成功」を無条件には保証しない。

環境によって、

- symbol duration
- frequency allocation
- repetition
- FEC
- interleave depth

を変える。

一瞬の会話でburst errorが起きても、Interleavingでエラーを分散し、FECで復元できる設計を目指す。

---

# 28. Recording Workflow

Logiscore内で直接録音する必要はない。

以下のような外部録音を受け付ける。

- iPhone Voice Memos
- Android Recorder
- PC Recorder
- OBS
- online call recording

フロー：

```text
Logiscore Music
      ↓
External Recording
      ↓
recording.m4a / wav / mp3
      ↓
Upload to Logiscore
      ↓
Signal Detection
      ↓
Decode
      ↓
FEC
      ↓
Password (optional)
      ↓
Decrypt
      ↓
Decompress
      ↓
Original Payload
```

---

# 29. Online Transmission

オンライン経由も評価対象とする。

段階：

```text
PCM
 ↓
WAV
 ↓
Opus
 ↓
WebRTC
 ↓
Real communication service
```

Online Profileでは音声通話向け処理による信号変形を考慮する。

---

# 30. テスト戦略

## Level 0 — Pure Digital

```text
Encoder
→ PCM
→ Decoder
```

成功条件：

```text
Recovered Payload == Original Payload
```

## Level 1 — File Codec

- WAV
- AAC
- MP3
- Opus

## Level 2 — Artificial Noise

- white noise
- pink noise
- speech
- music
- gain variation
- clipping
- room impulse response

## Level 3 — Network

- packet loss
- jitter
- resampling
- bitrate variation
- Opus encode/decode

## Level 4 — Physical

例：

```text
MacBook speaker
     ↓
room
     ↓
iPhone Voice Memo
     ↓
Logiscore
```

条件：

- 1m
- 3m
- 5m
- quiet
- conversation
- TV
- café-like noise
- reverberant room

---

# 31. 評価指標

通信：

```text
Source Size
Compressed Size
Header Size
FEC Overhead
Playback Duration
Payload Bitrate
Raw BER
Raw Symbol Accuracy
Corrected Errors
Final Recovery Rate
```

音楽：

```text
Average Melodic Leap
Voice Crossing Count
Strong-beat Non-Chord Notes
Roughness
Consonance Score
Polyphony
```

環境：

```text
Distance
SNR
Codec
Noise Type
Reverberation
Device
```

---

# 32. 大規模Payload

大規模プロジェクトでは再生時間が長くなる。

これは仕様上許容する。

UIでは事前に推定時間を表示する。

例：

```text
Estimated Transmission Time

Fast
18 min

Balanced
31 min

Robust
57 min
```

巨大リポジトリなら、

```text
Estimated duration
4h 37m
```

となってもよい。

この場合、

> 「不可能ではない。ただし音響帯域なので時間がかかる。」

ことを明示する。

---

# 33. 実装フェーズ

## Phase 1 — Protocol Cleanup

- Payload Type追加
- Text対応
- Header再設計
- Binary Header
- Transport abstraction
- Existing Dense Mode維持

## Phase 2 — Musical Baseline

- Tonal Context
- Chord Planner
- Candidate Generator
- 4 Voice Architecture
- Voice Leading
- Consonance Scoring

## Phase 3 — Musical Density

- Rhythm Encoding
- Multiple Symbol Channels
- Dynamic Candidate Count
- Bitrate optimization

## Phase 4 — Audio Decoder

- PCM generation
- STFT / FFT / Goertzel
- symbol detection
- timing recovery
- preamble

## Phase 5 — Error Correction

- CRC
- FEC
- Interleaving
- repetition

## Phase 6 — Physical Acoustic

- speaker → microphone
- smartphone recording
- WAV / M4A
- distance
- speech noise

## Phase 7 — Adaptive Profiles

- Auto
- Quiet
- Conversation
- Noisy
- Online
- Long Distance

## Phase 8 — Secure Mode

- Argon2id
- ChaCha20-Poly1305 / AES-GCM
- Password UX
- Authentication failure handling

## Phase 9 — Benchmark

以下のトレードオフを定量化する。

```text
Music Quality
vs
Bitrate
vs
Playback Time
vs
Noise Robustness
vs
Distance
```

---

# 34. 成功条件

最低限：

- MIDI round-trip 100%
- Text round-trip 100%
- File round-trip 100%

次段階：

- PCM round-trip 100%
- WAV round-trip 100%
- Opus round-trip 100%

Physical：

- smartphone recordingから復元
- quiet roomで安定
- conversation環境で高成功率
- FEC後の最終Payload完全一致

---

# 35. 今回の非目標

今回の改善では以下は主目的としない。

- Wi-FiやGitHubの代替
- あらゆる環境での100%保証
- 無制限距離
- 巨大データの高速伝送
- AIによる非決定論的作曲
- 映像チャネル

---

# 36. 将来的な構想 — Visual / Multimodal Codec

今回の実装範囲には含めないが、将来的には音だけでなく映像も情報チャネルとして利用したい。

候補：

- Pixel Art
- Animated Pixel Art
- Pixel Video
- 色
- 明度
- 座標
- frame transition
- movement
- pattern density

---

## Audio + Visual

```text
Source
  ↓
Unified Bitstream
  ↓
Adaptive Multiplexer
   /            \
Audio           Visual
Codec           Codec
 ↓               ↓
Music         Pixel Video
   \             /
    \           /
  Synchronized Media
         ↓
Capture / Recording
         ↓
Multimodal Decoder
         ↓
Original Payload
```

---

## 情報量に応じた映像変化

単純に映像にもデータを載せるだけではなく、情報量そのものを表現へ反映させる。

例えば、

```text
Low data density
→ simple pixel art
→ slow movement
→ low visual complexity
```

```text
High data density
→ more pixels
→ faster movement
→ richer color / pattern changes
```

といった形で、

> **情報量が増えるほど、音楽や映像の密度も変化する**

表現を検討する。

---

## 動的な帯域配分

AudioとVisualで固定比率にせず、その瞬間の条件に応じて情報量を調整する。

例：

```text
Audio capacity ↓
Visual capacity ↑
```

```text
Audio capacity ↑
Visual capacity ↓
```

これにより、音楽として自然に保ちたい区間では映像側へ情報を逃がすことも可能になる。

---

# 37. 長期的なLogiscoreの姿

現在：

> **Code to Music**

次段階：

> **Reversible Musical Codec**

さらに：

> **Acoustic Data Transport**

最終的には、

> **情報を、音・映像・時間構造を持つメディアへ可逆変換する Multimodal Codec**

へ発展できる。

ただし、まずは音楽単独で、

- 可逆性
- 音楽性
- 情報密度
- 録音耐性
- ノイズ耐性
- 環境適応

を成立させる。

Visual Codec はその基盤が安定した後の将来構想とする。
