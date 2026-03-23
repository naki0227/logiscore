# 🎵 Logiscore — Code to Music

> Transform your source code into orchestral MIDI music — entirely in the browser.

[![Live Demo](https://img.shields.io/badge/Live_Demo-logiscore.enludus.com-blue?style=for-the-badge)](https://logiscore.enludus.com)

## ✨ What is Logiscore?

**Logiscore** is a browser-based web application that converts source code projects into MIDI orchestral symphonies. Drag and drop a folder of source code, and watch — and listen — as each programming language becomes a unique instrument in your personal orchestra.

### 🔒 Zero-Trust: Your Code Never Leaves Your Machine

All processing happens **100% client-side** using Rust compiled to WebAssembly. No files are uploaded to any server, ever. Safe for proprietary and private codebases.

## 🎻 Language → Instrument Mapping

| Extension | Instrument | Why |
|-----------|-----------|-----|
| `.ts` / `.js` | Violin | The lead melody of web development |
| `.py` | Flute | Light and readable |
| `.rs` | Cello | Heavy and robust |
| `.go` | French Horn | The power of concurrency |
| `.css` | Harp | Adding beauty |
| `.html` | Piano | The foundation of everything |
| `Dockerfile` | Tubular Bells | A blessing for infrastructure |

## 🚀 Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| Frontend | React + TypeScript + Vite | UI, file input, playback |
| Engine | Rust → wasm-pack → WASM | Source → MIDI encoding |
| Audio | Web Audio API | In-browser playback |
| Hosting | Vercel (Edge Network) | Global delivery + SSL |

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

## 📄 License

MIT

---

**Try it now →** [logiscore.enludus.com](https://logiscore.enludus.com)
