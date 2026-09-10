# ADR 0008: PCM16 WAVとWeb AudioをPhysical Acoustic境界にする

- Status: Accepted
- Date: 2026-09-09

## 背景

Phase 5まででclean PCMと誤り訂正は成立したが、ユーザーが再生・録音・ファイル共有できる標準音声コンテナと、スマートフォン等の異なるsample rateを受け取る経路がなかった。

## 課題

- ブラウザから再生・保存できるlossless音声を生成する。
- 8 kHz以外、stereo、録音codecから得たPCMをdecoder入力へ正規化する。
- 壊れた音声、過大file、非有限sampleを安全に拒否する。
- 外部レコーダーを使うworkflowを複雑な設定なしで提供する。
- 信号劣化を実機試験前に自動再現できるようにする。

## 選択肢

1. WAV専用crateとresampling crateを追加する。
2. すべての音声処理をTypeScript/Web Audioだけへ置く。
3. PCM16 WAVとresamplingをRust coreに置き、圧縮音声のcontainer decodeだけWeb Audioへ委譲する。

## 採用した案

選択肢3を採用する。

- exportは8 kHz、mono、16-bit PCMのRIFF/WAVEとする。
- importはRustでPCM16およびfloat32 WAV、1〜32 channels、最大384 kHzを検証・mono化する。
- 異なるsample rateはRustのlinear interpolationで8 kHzへ正規化する。
- WAV以外のM4A、MP3、Opus、Ogg、AACはブラウザの`decodeAudioData`でPCM化し、同じRust decoderへ渡す。
- 音声fileは64 MiB、decode後は32,000,000 source frames、core入力は16,000,000 target samplesへ制限する。
- Reliable modeをWeb UIで有効化し、WAV生成、再生、download、録音file importを提供する。
- gain、deterministic noise、clipping、leading offset、single echoを`ChannelModel`で合成し、自動testに使用する。
- ブラウザworkflowはPlaywrightでWAV download/re-import、Web Audio PCM経路、破損file拒否まで自動実行する。

## 採用理由

WAVのwire検証とresamplingをRustへ集約するとnative/WASMで挙動を揃えられる。一方、M4A等のcontainer/codec対応を自前実装せずWeb Audioへ委譲することで、依存・WASM容量・codec保守を抑えながら一般的な録音fileを扱える。

## メリット

- 生成WAVは外部player・recorderで扱える標準形式になる。
- stereo、48 kHz等の録音を同じ8 kHz decoderへ渡せる。
- Web UIだけでText、Source File、ProjectのWAV共有を完結できる。
- coreの厳密なRIFF/chunk/format/size検証を再利用できる。
- 人工channel条件をCIで再現できる。
- 新規依存を追加しない。

## デメリット

- linear resamplingには高度なanti-alias filterがない。
- Web Audioの圧縮codec対応はOS・ブラウザに依存する。
- WAVは非圧縮なのでfile sizeが大きい。
- 物理距離・端末・部屋ごとの復元率は実機benchmarkが必要である。
- browser main threadで長いWASM処理を行うため、大規模payloadではUI停止時間が増える。

## 将来的な見直し条件

- 高品質resamplerが復元率を有意に改善する場合。
- 特定codecを全browserで同一にdecodeする必要が生じた場合。
- streaming encode/decodeまたはWeb Workerが必要になった場合。
- Phase 9の実機benchmarkで周波数・duration・channel modelを変更する場合。
