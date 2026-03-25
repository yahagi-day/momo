# MOMO - Multi-Output Media Orchestrator

Live video splitter/router that captures a single video input (DeckLink / UVC / Mock test pattern), applies per-output GPU transforms (crop → scale → flip), and outputs to multiple DeckLink devices simultaneously. A built-in web UI provides configuration, preview, and monitoring.

## System Overview

```
[DeckLink/UVC/Mock Input]
    │
    ▼
[Processing Engine (GPU: CUDA)]
    │  crop → scale → flip
    │
    ├→ [DeckLink Output 1]  per-output settings
    ├→ [DeckLink Output 2]  per-output settings
    ├→ ...N outputs          per-output settings
    │
    ├→ [Web Preview (MJPEG)]
    │
[Web UI (Browser)]
    │  config / control / preview
    │
[JSON Config File]
    │  persist / restore on startup
```

## Build & Run

```bash
cargo build                          # build all crates (CPU fallback)
cargo build --features gpu           # build with CUDA GPU processing (requires CUDA Toolkit)
cargo build --features decklink      # build with DeckLink hardware support (requires DeckLink SDK)
cargo build --features webrtc        # build with WebRTC preview support
cargo build --features gpu,decklink  # build with all hardware features
cargo run                            # start server (default: 0.0.0.0:8080)
cargo run -- --config path.json --port 9090
cargo test                           # run all tests
cargo clippy -- -D warnings          # lint
```

Frontend (optional — a fallback UI is embedded in the binary):

```bash
cd frontend && npm install && npm run build   # build SolidJS SPA
cd frontend && npm run dev                    # dev server with proxy to :8080
```

## Project Structure

```
momo/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── momo-core/              # shared types, config, error
│   ├── momo-decklink/          # DeckLink C++ FFI (cxx crate)
│   ├── momo-uvc/               # UVC input (v4l2 / MediaFoundation)
│   ├── momo-gpu/               # CUDA kernels + GPU memory management
│   │   └── kernels/            # crop.cu, scale.cu, flip.cu
│   ├── momo-webrtc/            # WebRTC preview (str0m + OpenH264)
│   ├── momo-pipeline/          # frame routing: input → GPU → outputs
│   ├── momo-web/               # axum REST API + WebSocket + MJPEG/WebRTC preview
│   └── momo-app/               # binary entry point
└── frontend/                   # SolidJS + Vite
```

| Crate | Role |
|---|---|
| `momo-core` | Shared types: `Frame`, `Config`, `Error`. JSON config serde + validation |
| `momo-decklink` | DeckLink SDK FFI. `VideoInput`/`VideoOutput` traits, device enumeration |
| `momo-uvc` | UVC camera input (Linux: v4l2, Windows: MediaFoundation) |
| `momo-gpu` | CUDA context management, crop/scale/flip kernels (PTX) |
| `momo-webrtc` | WebRTC preview streaming with H.264 encoding (str0m + OpenH264) |
| `momo-pipeline` | Frame routing: input → preview → N outputs. Mock input, preview encoding |
| `momo-web` | axum REST API + WebSocket + MJPEG/WebRTC preview. UI embedded at compile time |
| `momo-app` | CLI binary (clap). Wires all crates together |

## Input

- Sources: DeckLink, UVC, or Mock (color bar test pattern)
- Single input at a time
- Resolution and frame rate: configurable per source

## Output

Per-output settings for each DeckLink device:

- **Full frame**: pass-through
- **Crop**: pixel coordinates (x, y, width, height)
- **Scale**: resize to output format after crop
- **Flip**: horizontal / vertical
- **Format**: resolution and FPS per output
- **Scalable**: designed for 5+ simultaneous outputs

## Processing Pipeline

```
Input(CPU) → GPU upload → [Output 1: crop→scale→flip (CUDA stream 1)] → D2H → DeckLink 1
                        → [Output 2: crop→scale→flip (CUDA stream 2)] → D2H → DeckLink 2
                        → [Output N: ...]                              → ...
                        → [Preview: UYVY→RGB→scale→JPEG]               → MJPEG stream
                        → [Preview: UYVY→NV12→H.264]                   → WebRTC stream
```

- GPU processing via CUDA (GTX 1080+)
- All outputs share a single GPU source buffer with parallel CUDA streams
- Rotation (90° / 180° / 270°) planned for future

## API

```
GET    /api/devices              Device list + status
GET    /api/config               Current configuration
PUT    /api/config               Apply full configuration
PUT    /api/config/output/:id    Update single output transform
POST   /api/config/save          Persist to JSON file
POST   /api/config/load          Restore from JSON file
GET    /api/status               Pipeline state
POST   /api/pipeline/start       Start pipeline
POST   /api/pipeline/stop        Stop pipeline
GET    /api/preview/input        Input MJPEG stream
GET    /api/preview/output/:id   Per-output MJPEG stream (transformed)
WS     /ws/status                WebSocket: state changes, FPS, device events
WS     /ws/preview               WebRTC signaling (feature-gated: webrtc)
```

## Web UI

- **Status bar**: pipeline state, FPS display, start/stop control
- **Input panel**: source info + live MJPEG preview
- **Output cards**: per-output transform settings (crop, flip) with apply button + live output preview thumbnail
- **Config actions**: save/load configuration files
- **Real-time updates**: WebSocket-driven state and FPS
- **WebRTC preview**: Low-latency H.264 video preview (with `webrtc` feature), MJPEG fallback
- **FPS chart**: Real-time FPS visualization
- **Waveform**: Video-driven waveform analyzer

The UI HTML is embedded into the binary at compile time. If the SolidJS frontend is built (`frontend/dist/`), that is used; otherwise a self-contained fallback HTML is embedded.

## Platforms

- Ubuntu
- Windows

## Tech Stack

| Component | Technology | Rationale |
|---|---|---|
| Language | Rust | Safety + performance |
| DeckLink FFI | `cxx` crate | Safer than bindgen for COM-like C++ APIs |
| GPU | `cudarc` + nvcc PTX | Safe CUDA from Rust, kernels loaded as PTX |
| Web | axum + SolidJS | axum is tokio-native; SolidJS for reactive real-time UI |
| Preview | MJPEG + WebRTC (H.264) | MJPEG for compatibility, WebRTC for low-latency (feature-gated) |
| WebRTC | str0m + OpenH264 | Pure-Rust WebRTC stack, H.264 encoding via OpenH264 |
| Thread bridging | crossbeam-channel | Connects video OS threads with tokio async runtime |
| Config | serde_json | JSON serialization/deserialization |
