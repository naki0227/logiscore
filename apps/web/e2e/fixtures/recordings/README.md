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

1. Generate and play the Text payload in Reliable mode.
2. Place the iPhone microphone 1 m from the MacBook speaker.
3. Record one complete playback in Voice Memos, including silence before and
   after the signal.
4. Export the original M4A into this directory.
5. Fill `device_model`, an ISO-8601 `recorded_at`, and `recording_file` in
   `corpus.json`; change `status` from `planned` to `captured`.
6. Run `npm run test:e2e -- recording-corpus.spec.ts`. A captured entry is
   always imported and decoded by Playwright; manual browser confirmation does
   not count.
7. Populate all six `measurement` fields and change the status to `verified`
   only after the benchmark report is generated.

Never replace a failed recording with a synthetic file. Keep failed physical
conditions in the corpus so the performance boundary remains measurable.
