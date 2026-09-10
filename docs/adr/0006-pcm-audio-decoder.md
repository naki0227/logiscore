# ADR 0006: Rhythmic Musical Codecをclean PCMで検出する

- Status: Accepted
- Date: 2026-09-08

## 背景

Phase 3まではMIDI eventを直接復号しており、実音声を経由するAcoustic Transportの基準実装がなかった。Phase 4では後続の誤り訂正・録音試験を独立して進められるよう、決定的なPCM生成と復号境界が必要になる。

## 課題

- PCM内で開始位置、packet長、4声Pitch、Rhythm durationを復元する。
- MIDI metadataなしでframingを成立させる。
- 無制限入力、非有限値、同期誤検出を安全に拒否する。
- Text、Source File、Projectで同じv2 packetを再利用する。

## 選択肢

1. 汎用FFT/STFT依存ライブラリを追加する。
2. ブラウザのWeb Audio APIだけで検出する。
3. Rust coreに対象周波数限定のGoertzel detectorを実装する。

## 採用した案

選択肢3を採用する。

- 8 kHz mono `f32` PCMを基準入力とする。
- 96/84 MIDI noteのtwo-tone preambleで同期する。
- 88/92 MIDI noteの32 symbolsでpacket byte長をMSB firstに送る。
- Phase 3の4声Pitch候補と4種類のdurationをそのままPCM化する。
- onsetは振幅閾値、tone endは連続silence、Pitchは候補音に対するGoertzel energyで検出する。
- packetを4 KiB、PCM入力を16,000,000 samplesに制限する。
- `v2_packet`のtransport非依存packet build/decodeをPCMとMIDIで共有する。
- Rust APIをWASMへ公開し、Playwrightがブラウザ内で全payloadをround-tripする。

## 採用理由

Goertzelは検出対象周波数が候補音に限定される今回の用途で、FFT依存を増やさず実装・テストできる。信号処理をRust coreへ置くことでブラウザ以外でも同じ結果になり、将来のWAV・録音入力にも再利用できる。

## メリット

- 新規依存なしで決定的なPCM round-tripを得られる。
- MIDI metadataに依存せずpacket長を復元できる。
- musical domain、packet、audio detectionの責務が分離される。
- low deterministic noiseと先頭offsetを許容する基準ができる。
- WASM境界をPlaywrightで継続的に検証できる。

## デメリット

- sample rateは8 kHz固定で、resamplingは未対応。
- 固定閾値であり、gain変動・clipping・反響への適応は未実装。
- clean synthesized sine waveを対象とし、実スピーカー・マイクは未検証。
- FEC前なのでsymbol誤りを訂正できない。
- 4 KiB上限でも音響伝送時間は長い。

## 将来的な見直し条件

- WAV/M4Aや異なるsample rateを入力するとき。
- AGC、反響、圧縮codecで固定閾値が不十分なとき。
- Phase 5のFEC framingをpacketに導入するとき。
- adaptive frequency/profile選択を追加するとき。
