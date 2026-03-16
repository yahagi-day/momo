# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**momo** is a Rust-based live video splitter/router. It captures a single video input (DeckLink, UVC, or Mock test pattern), applies per-output GPU transformations (crop → scale → flip), and outputs to multiple DeckLink devices simultaneously. A web UI provides configuration, preview, and monitoring.

## Build & Test

```bash
cargo build                          # build all crates
cargo test                           # run all tests
cargo test -p momo-core              # run tests for a single crate
cargo test config_serde_roundtrip    # run a single test by name
cargo clippy -- -D warnings          # lint (treat warnings as errors)
cargo run                            # start the server (default: 0.0.0.0:8080)
cargo run -- --config path.json --port 9090  # custom config and port
```

Frontend (requires Node.js, optional — fallback HTML is embedded without it):

```bash
cd frontend && npm install && npm run build  # build SolidJS SPA into frontend/dist/
cd frontend && npm run dev                   # dev server with proxy to :8080
```

## Architecture

Cargo workspace with 7 crates. Dependency flow is strictly one-directional:

```
momo-core          shared types (Frame, Config, Error, PixelFormat, DisplayMode, etc.)
  ↑
  ├── momo-decklink   DeckLink FFI — VideoInput/VideoOutput traits, device enumeration (stub)
  ├── momo-uvc        UVC camera input (stub)
  ├── momo-gpu        CUDA kernels (stub)
  │
  ├── momo-pipeline   frame routing: input → GPU → N outputs (uses decklink, uvc, gpu)
  │     ↑
  │     └── momo-web  axum REST API + WebSocket + MJPEG preview
  │           ↑
  └───────── momo-app  binary entry point (clap CLI, tracing init, wires everything)
```

**Threading model**: Mock/hardware input runs on a dedicated OS thread. A bridge task (`tokio::task::spawn_blocking`) forwards frames from `crossbeam-channel` to `tokio::sync::mpsc` (100ms recv timeout). Preview encoding and FPS tracking run as a single tokio task. Web/API runs on the tokio async runtime.

**Frame flow**: Input thread → crossbeam channel (bounded 4) → bridge task → tokio mpsc (bounded 2) → preview task (UYVY→RGB→scale→JPEG via `spawn_blocking`) → `broadcast::Sender<Vec<u8>>` (capacity 4) → MJPEG endpoint.

**Event flow**: Pipeline state changes and FPS updates → `broadcast::Sender<PipelineEvent>` (capacity 64) → WebSocket handler forwards as JSON.

**UI embedding**: `momo-web/build.rs` embeds UI HTML at compile time via `include_str!`. If `frontend/dist/index.html` exists (from Vite + `vite-plugin-singlefile` build), it uses that; otherwise embeds `src/fallback.html`. The binary is fully self-contained — no external files needed.

**Pipeline lifecycle**: `Pipeline` is held in `Arc<RwLock<Pipeline>>` via `AppState`. `start()` creates `InputDriver::from_config()` → spawns input thread → bridge task → preview task. `stop()` sets `AtomicBool` stop flag, aborts bridge and preview tasks. The input thread checks the stop flag each frame and exits.

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| momo-core | **Complete** | All types, config, error, frame |
| momo-decklink | **Stub** | Traits defined, `enumerate_devices()` returns empty |
| momo-uvc | **Stub** | `enumerate_devices()` returns empty |
| momo-gpu | **Stub** | `is_cuda_available()` returns false |
| momo-pipeline | **Working** | Mock input, preview encode, FPS tracking |
| momo-web | **Working** | All endpoints, WebSocket, MJPEG, embedded UI |
| momo-app | **Working** | CLI, config loading, default config generation |
| frontend | **Working** | SolidJS SPA + vanilla JS fallback |

**Next steps**: Implement DeckLink FFI (Phase 1), CUDA GPU processing (Phase 2), UVC camera input (Phase 3).

## Workspace Dependencies

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio = { version = "1", features = ["full"] }
axum = "0.8"                                                    # ws feature added in momo-web
crossbeam-channel = "0.5"
clap = { version = "4", features = ["derive"] }
tower-http = { version = "0.6", features = ["cors"] }
image = { version = "0.25", default-features = false, features = ["jpeg"] }
async-stream = "0.3"
```

Dev dependencies (momo-web only): `tower = "0.5"` (util), `http-body-util = "0.1"` for endpoint testing via `tower::ServiceExt::oneshot`.

## Key Types

### momo-core

- **`Config`** — top-level config: `input: InputSource`, `outputs: Vec<OutputConfig>`, `preview: PreviewConfig`, `web: WebConfig`. Methods: `load(path)`, `save(path)`, `from_json(str)`, `to_json()`, `validate()`. Validation: ≥1 output, no duplicate IDs, non-zero crop dimensions.
- **`InputSource`** — tagged enum (`#[serde(tag = "type")]`): `DeckLink { device_index, display_mode, pixel_format }`, `Uvc { device_path }`, `Mock { width, height, fps }`.
- **`OutputConfig`** — `id`, `name`, `device_index`, `display_mode`, `pixel_format`, `transform: OutputTransform`, `enabled` (default true).
- **`OutputTransform`** — `crop: Option<CropRegion>`, `flip: FlipOptions` (horizontal/vertical bools).
- **`PreviewConfig`** — `width` (640), `height` (360), `fps` (10), `jpeg_quality` (75).
- **`Frame`** — `data: Vec<u8>`, `resolution: Resolution`, `format: PixelFormat`, `timestamp_ns: u64`, `sequence: u64`.
- **`PixelFormat`** — `Uyvy` (2 bytes/px), `Bgra` (4 bytes/px), `V210` (10-bit).
- **`DisplayMode`** — 19 DeckLink modes (720p/1080i/1080p/4K at various rates). Methods: `resolution()`, `frame_rate()`.
- **`PipelineState`** — `Stopped`, `Starting`, `Running`, `Stopping`, `Error`.
- **`Error`** — `DeckLink`, `Uvc`, `Gpu`, `Pipeline`, `Config`, `DeviceNotFound`, `DeviceDisconnected`, `Io`, `Json`.

### momo-decklink

- **`DeckLinkDevice`** — `index: u32`, `name: String`, `status: DeviceStatus`.
- **`VideoInput`** trait (Send) — `start()`, `stop()`, `is_capturing()`.
- **`VideoOutput`** trait (Send) — `start()`, `stop()`, `send_frame(&Frame)`.
- **`enumerate_devices()`** — currently returns empty Vec.

### momo-pipeline

- **`Pipeline`** — fields: `state`, `config: Option<Config>`, `config_path: Option<PathBuf>`, `event_tx: broadcast::Sender<PipelineEvent>`, `preview_tx: broadcast::Sender<Vec<u8>>`, `running: Option<RunningState>`. Methods: `new()`, `state()`, `config()`, `subscribe()`, `subscribe_preview()`, `set_config()`, `set_config_path()`, `start()`, `stop()`, `save_config()`, `load_config()`, `update_output()`, `outputs()`.
- **`PipelineEvent`** — `StateChanged { state }`, `FpsUpdate { fps: f64 }`, `DeviceEvent { device, status }`, `ConfigChanged`, `Error { message }`. Serialized as tagged JSON.
- **`InputDriver`** — enum: `Mock(MockInput)`. Factory: `from_config(&InputSource)`. Non-Mock sources return error.
- **`MockInput`** — generates UYVY color bars (SMPTE 8-bar pattern: White/Yellow/Cyan/Green/Magenta/Red/Blue/Black). Runs on dedicated OS thread at configured FPS. Stop via `Arc<AtomicBool>`.
- **`preview.rs`** — `uyvy_to_rgb()` (BT.601 coefficients), `nearest_neighbor_scale()`, `encode_preview()` (→ JPEG via `image::codecs::jpeg::JpegEncoder`).

### momo-web

- **`AppState`** — `pipeline: Arc<RwLock<Pipeline>>`. Constructor: `new(pipeline)`.
- **`AppError`** — wraps `momo_core::Error`. Maps: Config/Json→400, DeviceNotFound→404, Pipeline→409, others→500.
- **`embedded_ui::index_handler()`** — serves `include_str!(concat!(env!("OUT_DIR"), "/index.html"))`.

### momo-app

- **CLI args** (clap): `--config` (default "config.json"), `--bind` (default "0.0.0.0"), `--port` (default 8080).
- **Default config**: Mock 1920×1080 @ 30fps, one output "out1" Hd1080p5994 Uyvy.
- **Startup**: loads config file if exists, otherwise creates default Mock config with warning.

## API Endpoints

```
GET    /api/status                → { state: PipelineState }
POST   /api/pipeline/start        → { status: "ok" } | 409
POST   /api/pipeline/stop         → { status: "ok" } | 409
GET    /api/config                → Config JSON | 400 (no config)
PUT    /api/config                → { status: "ok" } (sets full config)
PUT    /api/config/output/{id}    → { status: "ok" } (updates OutputTransform)
POST   /api/config/save           → { status: "ok" } (saves to config_path)
POST   /api/config/load           → { status: "ok" } (body: { path: "..." })
GET    /api/devices               → DeviceInfo[] (currently empty)
GET    /api/preview/input         → multipart/x-mixed-replace MJPEG stream
GET    /api/preview/output/{id}   → 501 Not Implemented (stub)
WS     /ws/status                 → PipelineEvent JSON messages
```

## Tests (30 total)

**momo-core** (7): `config_serde_roundtrip`, `config_file_roundtrip`, `config_rejects_empty_outputs`, `config_rejects_duplicate_ids`, `config_rejects_zero_crop`, `config_defaults_applied`, `input_source_uvc_roundtrip`.

**momo-pipeline** (13): `color_bar_frame_size`, `color_bar_frame_small`, `initial_state_is_stopped`, `set_config`, `update_output_transform`, `update_output_not_found`, `start_stop_lifecycle`, `start_without_config_fails`, `stop_when_stopped_fails`, `subscribe_preview` (waits 3s for JPEG frame), `uyvy_to_rgb_known_values`, `nearest_neighbor_scale_halves`, `encode_preview_produces_jpeg` (checks FFD8 magic bytes).

**momo-web** (10): `get_status_returns_stopped`, `get_config_returns_config`, `get_config_no_config_returns_400`, `put_config_sets_config`, `get_devices_returns_array`, `start_stop_pipeline`, `stop_when_stopped_returns_conflict`, `preview_output_returns_501`, `preview_input_returns_mjpeg_content_type`, `update_output_transform`.

## Frontend (SolidJS + Vite)

`frontend/` — SolidJS SPA with TypeScript. Uses `vite-plugin-singlefile` to produce a single HTML file for binary embedding. Dev proxy: `/api` → `:8080`, `/ws` → `ws://:8080`.

**Components**: `App` (root, state management, WebSocket), `StatusBar` (state badge, FPS, start/stop), `InputPanel` (source label, MJPEG preview when running), `OutputList`/`OutputCard` (per-output flip controls with Apply), `PreviewImage` (img tag), `ConfigActions` (save/load buttons).

**API client** (`src/api/client.ts`): `getStatus`, `getConfig`, `putConfig`, `updateOutput`, `saveConfig`, `loadConfig`, `startPipeline`, `stopPipeline`.

**WebSocket** (`src/api/websocket.ts`): auto-reconnect with 2s delay.

## CI (GitHub Actions)

**check** job (ubuntu): Node 22 → `npm install && npm run build` → `cargo build` → `cargo clippy -- -D warnings` → `cargo test`.

**build** job (matrix: ubuntu + windows): same frontend build → `cargo build --release --target` → upload artifact (`momo-app` / `momo-app.exe`).

Frontend is built BEFORE cargo so that `build.rs` can embed `frontend/dist/index.html` into the binary.

## Key Constraints

- Live/broadcast use — low latency is critical, no allocations in hot path
- DeckLink SDK is C++; access via `cxx` crate FFI bindings (not bindgen)
- Must handle DeckLink hot-unplug gracefully (no crash, status reflected in UI)
- Cross-platform: Ubuntu and Windows
- GPU: CUDA via `cudarc`, targeting GTX 1080+
- DeckLink/CUDA/UVC are currently stubs — all tests must pass without hardware
- When `--config` file doesn't exist, a default Mock input config is generated automatically
- Config uses apply-button paradigm — changes are explicit, not auto-applied

## Language

- Code, comments, and commit messages: English
- UI text and documentation: Japanese is acceptable
