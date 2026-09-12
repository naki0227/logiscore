# Smartphone recording corpus

The first physical condition is fixed:

```text
MacBook Speaker
      ↓
     1 m
      ↓
iPhone Voice Memos
      ↓
     M4A
      ↓
Logiscore Web Audio import
```

Use a quiet room and the Text payload `Hello from Logiscore.`. Do not trim,
normalize, transcode, or rename the Voice Memos export before preserving the
original file.

## Capture workflow

1. Play the committed `capture-text-fixed-fallback.wav` from beginning to end.
   It contains the exact Text payload above in FixedFallback profile, PCM16 mono
   8 kHz, and lasts about 136.72 seconds.
2. Place the iPhone microphone 1 m from the MacBook speaker.
3. Record one complete playback in Voice Memos, including silence before and
   after the signal.
4. Export the original M4A into this directory.
5. Fill `device_model`, an ISO-8601 `recorded_at`, `recording_file`, and the
   original file's lowercase `recording_sha256` in `corpus.json`; change
   `status` from `planned` to `captured`.
6. Run `npm run test:e2e -- recording-corpus.spec.ts`. A captured entry is
   always imported and measured by Playwright, including failed recovery. The
   six-metric result is attached to the Playwright report as
   `recording-measurement.json`; manual browser confirmation does not count.
7. Populate all six `measurement` fields and change the status to `verified`
   only after the benchmark report is generated.

Never replace a failed recording with a synthetic file. Keep failed physical
conditions in the corpus so the performance boundary remains measurable.

The committed transmission fixture can be reproduced from the repository root:

```sh
cargo run --locked --example generate_recording_fixture -- \
  apps/web/e2e/fixtures/recordings/capture-text-fixed-fallback.wav \
  "Hello from Logiscore."
```

## Verified baseline

The first unmodified iPhone Voice Memos recording is
`iphone-record-1m-quiet-001.m4a`. Playwright restored the exact Text payload
through Web Audio with these stable measurements:

- Final Recovery Rate: 100%
- Raw Symbol Accuracy: 99.86462093862816%
- Corrected Errors: 3 bits
- Payload Bitrate: 1.2024851359476252 bps
- Playback Duration: 139710.66666666666 ms (AAC priming may vary by less than
  1 ms between browser decodes)

Decode Time is retained in `corpus.json` as an observed value but is not used
for strict regression comparison because it depends on the execution host.
