<p align="center">
  <br>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/-MOMO-ff69b4?style=for-the-badge&labelColor=1a1a2e">
    <img alt="MOMO" src="https://img.shields.io/badge/-MOMO-ff69b4?style=for-the-badge&labelColor=1a1a2e">
  </picture>
  <br>
</p>

```
  ███╗   ███╗  ██████╗  ███╗   ███╗  ██████╗
  ████╗ ████║ ██╔═══██╗ ████╗ ████║ ██╔═══██╗
  ██╔████╔██║ ██║   ██║ ██╔████╔██║ ██║   ██║
  ██║╚██╔╝██║ ██║   ██║ ██║╚██╔╝██║ ██║   ██║
  ██║ ╚═╝ ██║ ╚██████╔╝ ██║ ╚═╝ ██║ ╚██████╔╝
  ╚═╝     ╚═╝  ╚═════╝  ╚═╝     ╚═╝  ╚═════╝
```

<p align="center">
  <strong>Live video splitter/router built with Rust</strong>
  <br>
  <sub>1 input. N outputs. GPU-accelerated. Zero-copy.</sub>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#features">Features</a> &bull;
  <a href="#architecture">Architecture</a> &bull;
  <a href="#api">API</a> &bull;
  <a href="#web-ui">Web UI</a>
</p>

---

## What is MOMO?

MOMO captures a single video input (DeckLink / UVC / Mock test pattern), applies **per-output GPU transforms** (crop, scale, flip), and routes to multiple DeckLink outputs simultaneously — all from a single binary with a built-in web UI.

```
                  ┌─────────────────┐
                  │  DeckLink Input  │
                  │  UVC Camera      │
                  │  Mock Pattern    │
                  └────────┬────────┘
                           │
                    ┌──────▼──────┐
                    │  GPU Engine  │
                    │  crop→scale  │
                    │  →flip       │
                    └──┬───┬───┬──┘
                       │   │   │
              ┌────────┘   │   └────────┐
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ Output 1 │ │ Output 2 │ │ Output N │
        │ DeckLink │ │ DeckLink │ │ DeckLink │
        └──────────┘ └──────────┘ └──────────┘
              │            │            │
              └────────┬───┘────────────┘
                       ▼
                ┌─────────────┐
                │   Web UI    │
                │  MJPEG /    │
                │  WebRTC     │
                └─────────────┘
```

## Features

| | Feature | Details |
|---|---|---|
| **Input** | Multi-source | DeckLink, UVC camera, or Mock test pattern |
| **Output** | Multi-output | N simultaneous DeckLink outputs with independent settings |
| **Transform** | Per-output GPU | Crop, scale, flip per output via CUDA (GTX 1080+) |
| **Preview** | Dual streaming | MJPEG for compatibility + WebRTC (H.264) for low-latency |
| **Web UI** | Built-in | SolidJS SPA embedded in binary — no external files needed |
| **Config** | JSON-based | Hot-reloadable, apply-button paradigm |
| **Platform** | Cross-platform | Linux + Windows |

### Feature Flags

All hardware features are opt-in. The default build runs tests and preview with no hardware required.

```
--features decklink    DeckLink capture/output (requires SDK)
--features gpu         CUDA processing (requires CUDA Toolkit)
--features uvc         UVC camera input
--features webrtc      WebRTC preview (H.264 via OpenH264)
```

## Quick Start

```bash
# Clone & build
git clone https://github.com/yahagi-day/momo.git
cd momo
cargo build

# (Optional) Build the SolidJS frontend
cd frontend && npm install && npm run build && cd ..

# Run with default mock input
cargo run
# => http://localhost:8080
```

```bash
# With hardware features
cargo build --features gpu,decklink,webrtc
cargo run -- --config my-config.json --port 9090
```

## Architecture

Cargo workspace with 8 crates. Dependency flow is strictly one-directional:

```
momo-core             Shared types (Frame, Config, Error, PixelFormat, DisplayMode)
  ↑
  ├── momo-decklink   DeckLink FFI via cxx
  ├── momo-uvc        UVC camera input via nokhwa
  ├── momo-gpu        CUDA kernels + CPU fallback
  ├── momo-webrtc     WebRTC preview (str0m + OpenH264)
  │
  ├── momo-pipeline   Frame routing: input → GPU → N outputs
  │     ↑
  │     └── momo-web  axum REST API + WebSocket + preview
  │           ↑
  └───────── momo-app Binary entry point
```

| Crate | Role |
|---|---|
| `momo-core` | Shared types: `Frame`, `Config`, `Error`. JSON config serde + validation |
| `momo-decklink` | DeckLink SDK FFI. `VideoInput`/`VideoOutput` traits, device enumeration |
| `momo-uvc` | UVC camera input (Linux: v4l2, Windows: MediaFoundation) |
| `momo-gpu` | CUDA crop/scale/flip kernels with CPU fallback |
| `momo-webrtc` | WebRTC preview with H.264 encoding (str0m + OpenH264) |
| `momo-pipeline` | Frame routing, mock input, preview encoding, FPS tracking |
| `momo-web` | axum API + WebSocket + MJPEG/WebRTC preview, embedded UI |
| `momo-app` | CLI binary (clap), wires everything together |

### Processing Pipeline

```
Input (OS thread) → GPU upload → ┬─ [Output 1: crop→scale→flip] → DeckLink 1
                                  ├─ [Output 2: crop→scale→flip] → DeckLink 2
                                  ├─ [Output N: ...]              → DeckLink N
                                  ├─ [Preview: UYVY→RGB→JPEG]     → MJPEG stream
                                  └─ [Preview: UYVY→NV12→H.264]   → WebRTC stream
```

### Threading Model

- Hardware/mock input runs on a **dedicated OS thread**
- `crossbeam-channel` bridges to **tokio async runtime**
- Preview encoding + FPS tracking run as tokio tasks
- Zero-copy frame sharing via `Arc<Vec<u8>>`

## API

```
GET    /api/status               Pipeline state
POST   /api/pipeline/start       Start pipeline
POST   /api/pipeline/stop        Stop pipeline
GET    /api/config               Current configuration
PUT    /api/config               Apply full configuration
PUT    /api/config/output/:id    Update single output transform
POST   /api/config/save          Persist to JSON file
POST   /api/config/load          Restore from JSON file
GET    /api/devices              Device list + status
GET    /api/preview/input        Input MJPEG stream
GET    /api/preview/output/:id   Per-output MJPEG stream
WS     /ws/status                State changes, FPS, device events
WS     /ws/preview               WebRTC signaling (feature-gated)
```

## Web UI

The UI is **embedded into the binary** at compile time — no external files needed.

- **Status bar** — pipeline state, FPS display, start/stop
- **Input panel** — source info + live MJPEG/WebRTC preview
- **Output cards** — per-output crop/flip editing with visual crop overlay
- **FPS chart** — real-time FPS visualization
- **Waveform** — video-driven waveform analyzer
- **Config actions** — save/load configuration files

## Development

```bash
cargo test                           # run all tests (60 total)
cargo test -p momo-core              # single crate
cargo test config_serde_roundtrip    # single test
cargo clippy -- -D warnings          # lint (CI enforced)
```

```bash
# Frontend dev server (hot reload, proxied to :8080)
cd frontend && npm run dev
```

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| DeckLink FFI | `cxx` |
| GPU | `cudarc` + nvcc PTX |
| Web server | axum |
| Frontend | SolidJS + Vite |
| Preview | MJPEG + WebRTC (str0m + OpenH264) |
| Thread bridge | crossbeam-channel |
| Config | serde_json |

## License

See [LICENSE](LICENSE) for details.
