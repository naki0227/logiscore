# Recording fixtures

`logiscore-opus.ogg` is a deterministic Opus round-trip fixture for the text
`Opus`. It deliberately uses the monophonic `FixedFallback` acoustic profile:
the fixed symbol slots survive Opus pre-echo, while the rhythmic profiles still
need a more tolerant tone-boundary detector.

Regenerate and verify it from the repository root:

```sh
cargo run --release --locked --manifest-path packages/harmonic-core/Cargo.toml \
  --example generate_recording_fixture -- /tmp/logiscore-opus-fixture.wav
ffmpeg -y -i /tmp/logiscore-opus-fixture.wav -c:a libopus -b:a 128k -vbr off \
  -compression_level 10 -application lowdelay -frame_duration 5 -ar 48000 \
  apps/web/e2e/fixtures/logiscore-opus.ogg
ffmpeg -y -i apps/web/e2e/fixtures/logiscore-opus.ogg -ac 1 -ar 48000 \
  -c:a pcm_f32le /tmp/logiscore-opus-decoded.wav
cargo run --release --locked --manifest-path packages/harmonic-core/Cargo.toml \
  --example decode_recording_fixture -- /tmp/logiscore-opus-decoded.wav
```

The final command must print `Opus`. Browser behavior is covered by
`recording-formats.spec.ts`; run it through Playwright rather than checking the
page manually.
