# 🎵 Logiscore — Code to Music

> Transform your source code into orchestral MIDI music — entirely in the browser.

[![Live Demo](https://img.shields.io/badge/Live_Demo-logiscore.enludus.com-blue?style=for-the-badge)](https://logiscore.enludus.com)

## ✨ What is Logiscore?

**Logiscore** is a browser-based web application that converts source code projects into MIDI orchestral symphonies. Drag and drop a folder of source code, and watch — and listen — as each programming language becomes a unique instrument in your personal orchestra.

### 🔒 Zero-Trust: Your Code Never Leaves Your Machine

All processing happens **100% client-side** using Rust compiled to WebAssembly. No files are uploaded to any server, ever. Safe for proprietary and private codebases.

## 🎻 Language → Instrument Mapping

| Extension     | Instrument    | Why                                |
| ------------- | ------------- | ---------------------------------- |
| `.ts` / `.js` | Violin        | The lead melody of web development |
| `.py`         | Flute         | Light and readable                 |
| `.rs`         | Cello         | Heavy and robust                   |
| `.go`         | French Horn   | The power of concurrency           |
| `.css`        | Harp          | Adding beauty                      |
| `.html`       | Piano         | The foundation of everything       |
| `Dockerfile`  | Tubular Bells | A blessing for infrastructure      |

## 🚀 Tech Stack

| Layer    | Technology                | Role                     |
| -------- | ------------------------- | ------------------------ |
| Frontend | React + TypeScript + Vite | UI, file input, playback |
| Engine   | Rust → wasm-pack → WASM   | Source → MIDI encoding   |
| Audio    | Web Audio API             | In-browser playback      |
| Hosting  | Vercel (Edge Network)     | Global delivery + SSL    |

## ⚡ Key Optimizations

- **Hyper Diet MIDI**: Running Status + 127-delta VLQ constraint for ~40% binary size reduction
- **Smart Filtering**: Automatic `.gitignore` parsing to exclude `node_modules`, `.git`, and build artifacts
- **Sub-millisecond**: Handles 2,000+ files with negligible latency

## 🛠️ Local Development

```bash
# Prerequisites: Rust, wasm-pack, Node.js

# 1. Build WASM
cd packages/harmonic-core
wasm-pack build --target web --release --out-dir ../../apps/web/src/pkg

# 2. Start dev server
cd ../../apps/web
npm install
npm run dev
```

### Browser E2E and load scenarios

```bash
# Install the Chromium version matched to Playwright, then run browser scenarios
cd apps/web
npx playwright install chromium
npm run test:e2e

# In another terminal, start the production preview
npm run build
npm run preview -- --host 0.0.0.0 --port 4173

# Run the static-delivery load scenario without adding k6 to app dependencies
docker run --rm --network host \
  -v "$PWD/tests/load:/scripts:ro" \
  grafana/k6:2.2.0 run \
  -e BASE_URL=http://host.docker.internal:4173 \
  /scripts/static-assets.js
```

### Production deployment

The GitHub Actions `Vercel production` job runs only after both Rust and Web jobs succeed on `main`. Configure these repository Secrets:

- `VERCEL_TOKEN`
- `VERCEL_ORG_ID`
- `VERCEL_PROJECT_ID`

Then set the repository Variable `VERCEL_DEPLOY_ENABLED` to `true`. Keep it disabled while Vercel Git Integration is active to avoid duplicate deployments.

## 📄 License

MIT

---

**Try it now →** [logiscore.enludus.com](https://logiscore.enludus.com)

## v2 Development

v2 is now under active development as a transport-independent Musical Codec / Acoustic Transport. The existing v1 Dense MIDI codec remains supported while the new Payload, Binary Header, and Transport layers are introduced incrementally.

Text, single Source File, and Project payloads support both v2 Dense MIDI and the deterministic four-voice Rhythmic Musical MIDI profile. Pitch and rhythm jointly carry data while existing v1 and v2 Musical MIDI files remain import-compatible.

Phase 4 also provides an 8 kHz clean-PCM transport API for all three payloads. It adds a two-tone preamble, an explicit length frame, onset/timing recovery, and Goertzel-based chord detection. This is the deterministic audio-decoder baseline; speaker/microphone capture and recorded-file import remain Phase 6 work.

Reliable PCM uses v2 FEC profile `1`: CRC-32, Hamming SECDED, 8-codeword bit interleaving, and three-copy majority recovery. The original unprotected profile remains available and wire-compatible.

Reliable mode exports an 8 kHz mono PCM16 WAV that can be played, recorded externally, and imported again. PCM16/float32 WAV is decoded in Rust; other browser-supported recordings such as M4A, MP3, Opus, Ogg, and AAC are converted to mono PCM through Web Audio and resampled by the same Rust decoder.

- Design: [`docs/logiscore_improvement_design.md`](docs/logiscore_improvement_design.md)
- Current work: [`docs/TODO.md`](docs/TODO.md)
- Architecture decision: [`docs/adr/0001-v2-protocol-boundaries.md`](docs/adr/0001-v2-protocol-boundaries.md)
- Project archive decision: [`docs/adr/0002-v2-project-archive.md`](docs/adr/0002-v2-project-archive.md)
- Musical baseline: [`docs/adr/0003-musical-baseline.md`](docs/adr/0003-musical-baseline.md)
- Musical MIDI transport: [`docs/adr/0004-musical-midi-transport.md`](docs/adr/0004-musical-midi-transport.md)
- Rhythmic density: [`docs/adr/0005-rhythmic-musical-density.md`](docs/adr/0005-rhythmic-musical-density.md)
- PCM audio decoder: [`docs/adr/0006-pcm-audio-decoder.md`](docs/adr/0006-pcm-audio-decoder.md)
- Error-correction profile: [`docs/adr/0007-error-correction-profile.md`](docs/adr/0007-error-correction-profile.md)
- Physical audio workflow: [`docs/adr/0008-physical-audio-workflow.md`](docs/adr/0008-physical-audio-workflow.md)
- Adaptive profile selection: [`docs/adr/0009-adaptive-profile-selection.md`](docs/adr/0009-adaptive-profile-selection.md)
- Secure envelope: [`docs/adr/0010-secure-envelope.md`](docs/adr/0010-secure-envelope.md)
- Vercel deployment gate: [`docs/adr/0011-vercel-deployment-gate.md`](docs/adr/0011-vercel-deployment-gate.md)
