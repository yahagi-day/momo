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
cargo build --features decklink      # build with DeckLink hardware support
cargo build --features uvc           # build with UVC camera support
cargo build --features gpu           # build with CUDA GPU processing
cargo build --features webrtc        # build with WebRTC preview support
```

Frontend (requires Node.js, optional — fallback HTML is embedded without it):

```bash
cd frontend && npm install && npm run build  # build SolidJS SPA into frontend/dist/
cd frontend && npm run dev                   # dev server with proxy to :8080
```

## Architecture

Cargo workspace with 8 crates. Dependency flow is strictly one-directional:

```
momo-core          shared types (Frame, Config, Error, PixelFormat, DisplayMode, etc.)
  ↑
  ├── momo-decklink   DeckLink FFI via cxx — device enumeration, input capture, output (feature-gated)
  ├── momo-uvc        UVC camera input via nokhwa (feature-gated)
  ├── momo-gpu        GPU processing: crop/scale/flip with CPU fallback (CUDA feature-gated)
  ├── momo-webrtc     WebRTC preview via str0m + OpenH264 H.264 encoding (feature-gated)
  │
  ├── momo-pipeline   frame routing: input → GPU → N outputs (uses decklink, uvc, gpu)
  │     ↑
  │     └── momo-web  axum REST API + WebSocket + MJPEG/WebRTC preview (uses webrtc)
  │           ↑
  └───────── momo-app  binary entry point (clap CLI, tracing init, wires everything)
```

**Threading model**: Mock/hardware input runs on a dedicated OS thread. A bridge task (`tokio::task::spawn_blocking`) forwards frames from `crossbeam-channel` to `tokio::sync::mpsc` (100ms recv timeout). Preview encoding and FPS tracking run as a single tokio task. Web/API runs on the tokio async runtime.

**Frame flow**: Input thread → crossbeam channel (bounded 4) → bridge task → tokio mpsc (bounded 2) → [per-output: GpuProcessor::process() crop→scale→flip → DeckLink output (feature-gated) + per-output preview broadcast channel] + [input preview task (UYVY→RGB→scale→JPEG via `spawn_blocking`) → `broadcast::Sender<Vec<u8>>` (capacity 4) → MJPEG endpoint] + [raw frame broadcast → `broadcast::Sender<Arc<Frame>>` → WebRTC H.264 encoding (feature-gated)].

**Event flow**: Pipeline state changes and FPS updates → `broadcast::Sender<PipelineEvent>` (capacity 64) → WebSocket handler forwards as JSON.

**UI embedding**: `momo-web/build.rs` embeds UI HTML at compile time via `include_str!`. If `frontend/dist/index.html` exists (from Vite + `vite-plugin-singlefile` build), it uses that; otherwise embeds `src/fallback.html`. The binary is fully self-contained — no external files needed.

**Pipeline lifecycle**: `Pipeline` is held in `Arc<RwLock<Pipeline>>` via `AppState`. `start()` creates `InputDriver::from_config()` → spawns input thread → bridge task → preview task. `stop()` sets `AtomicBool` stop flag, aborts bridge and preview tasks. The input thread checks the stop flag each frame and exits.

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| momo-core | **Complete** | All types, config, error, frame |
| momo-decklink | **Working** | cxx FFI bridge, feature-gated (`--features decklink`). Input capture, output, device enumeration. Stub without feature. |
| momo-uvc | **Working** | Feature-gated (`--features uvc`). nokhwa capture, YUYV→UYVY conversion. Stub without feature. |
| momo-gpu | **Working** | CUDA kernels (crop/scale/flip) with CPU fallback. PTX compiled at build time by nvcc. Feature-gated (`--features gpu`). |
| momo-webrtc | **Working** | WebRTC preview streaming, H.264 via OpenH264 + str0m. Feature-gated (`--features webrtc`). Signal types always available. |
| momo-pipeline | **Working** | Mock input, preview encode, FPS tracking, per-output preview, raw frame broadcast for WebRTC, DeckLink output integration (feature-gated) |
| momo-web | **Working** | All endpoints, WebSocket, MJPEG input+output preview, WebRTC signaling WS (feature-gated), embedded UI |
| momo-app | **Working** | CLI, config loading, default config generation |
| frontend | **Working** | SolidJS SPA + vanilla JS fallback |

**Next steps**: Rotation support (90°/180°/270°), CUDA kernel optimization (persistent device buffers, parallel CUDA streams per output), DeckLink hot-unplug handling.

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
cxx = "1"
cudarc = { version = "0.16", features = ["driver", "cuda-version-from-build-system"] }
nokhwa = { version = "0.10", features = ["input-native"] }
clap = { version = "4", features = ["derive"] }
tower-http = { version = "0.6", features = ["cors"] }
image = { version = "0.25", default-features = false, features = ["jpeg"] }
async-stream = "0.3"
```

Crate-level dependencies (not workspace-shared): `str0m = "0.7"` and `openh264 = { version = "0.6", features = ["source"] }` in momo-webrtc (optional, behind `webrtc` feature).

Dev dependencies (momo-web only): `tower = "0.5"` (util), `http-body-util = "0.1"` for endpoint testing via `tower::ServiceExt::oneshot`.

Build dependencies (momo-decklink): `cxx-build = "1"` for compiling C++ bridge code when `decklink` feature is enabled. On Windows, also requires MSVC (`midl.exe`) for IDL compilation.

Build dependencies (momo-gpu): `build.rs` compiles `.cu` kernels to PTX via `nvcc` when `gpu` feature is enabled. Requires CUDA Toolkit.

## Key Types

### momo-core

- **`Config`** — top-level config: `input: InputSource`, `outputs: Vec<OutputConfig>`, `preview: PreviewConfig`, `web: WebConfig`. Methods: `load(path)`, `save(path)`, `from_json(str)`, `to_json()`, `validate()`. Validation: ≥1 output, no duplicate IDs, non-zero crop dimensions.
- **`InputSource`** — tagged enum (`#[serde(tag = "type")]`): `DeckLink { device_index, display_mode, pixel_format }`, `Uvc { device_path }`, `Mock { width, height, fps }`.
- **`OutputConfig`** — `id`, `name`, `device_index`, `display_mode`, `pixel_format`, `transform: OutputTransform`, `enabled` (default true).
- **`OutputTransform`** — `crop: Option<CropRegion>`, `flip: FlipOptions` (horizontal/vertical bools).
- **`PreviewConfig`** — `width` (640), `height` (360), `fps` (10), `jpeg_quality` (75).
- **`Frame`** — `data: Arc<Vec<u8>>`, `resolution: Resolution`, `format: PixelFormat`, `timestamp_ns: u64`, `sequence: u64`. Data wrapped in `Arc` for zero-copy sharing across broadcast channels.
- **`PixelFormat`** — `Uyvy` (2 bytes/px), `Bgra` (4 bytes/px), `V210` (10-bit).
- **`DisplayMode`** — 19 DeckLink modes (720p/1080i/1080p/4K at various rates). Methods: `resolution()`, `frame_rate()`.
- **`PipelineState`** — `Stopped`, `Starting`, `Running`, `Stopping`, `Error`.
- **`Error`** — `DeckLink`, `Uvc`, `Gpu`, `Pipeline`, `Config`, `DeviceNotFound`, `DeviceDisconnected`, `Io`, `Json`.

### momo-decklink

- **Feature flag**: `decklink` — enables C++ FFI via `cxx`. Default OFF (stub only). Propagates: `momo-app/decklink` → `momo-pipeline/decklink` → `momo-decklink/decklink`.
- **Cross-platform build** (`build.rs`): Linux uses `DeckLinkAPIDispatch.cpp` (dlopen) + SDK headers from `sdk/include/`. Windows uses `midl.exe` to generate C++ headers from IDL files in `sdk/include/win/`, links `ole32`/`oleaut32`.
- **SDK headers** (`sdk/include/`): Committed to git. Per EULA Section 0, Include headers are exempt from redistribution restrictions (Clauses 1, 4.3, 4.4, 5, 7, 8 do not apply). Windows IDL files in `sdk/include/win/` also committed. Other SDK files (samples, docs) are gitignored.
- **`DeckLinkDevice`** — `index: u32`, `name: String`, `model_name: String`, `has_input: bool`, `has_output: bool`, `status: DeviceStatus`.
- **`VideoInput`** trait (Send) — `start()`, `stop()`, `is_capturing()`.
- **`VideoOutput`** trait (Send) — `start()`, `stop()`, `send_frame(&Frame)`.
- **`enumerate_devices()`** — with `decklink` feature: queries hardware via COM. Without: returns empty Vec.
- **`conversions`** module — `DisplayMode` ↔ BMDDisplayMode (`u32`) and `PixelFormat` ↔ BMDPixelFormat (`u32`) conversions.
- **`ffi`** module (feature-gated) — `cxx::bridge` to C++ `DeckLinkSystem`, `DeckLinkInputCapture`, `DeckLinkOutputPlayer`.
- **`input::DeckLinkInput`** (feature-gated) — captures frames on dedicated OS thread via callback → mutex/CV → `get_frame()` polling loop → crossbeam channel.
- **`output::DeckLinkOutput`** (feature-gated) — implements `VideoOutput`, uses `DisplayVideoFrameSync` with 3-frame pool.
- **C++ bridge** (`cpp/decklink_bridge.cpp`) — wraps COM interfaces: `IDeckLinkIterator`, `IDeckLinkInput`+`IDeckLinkInputCallback`, `IDeckLinkOutput`+`IDeckLinkMutableVideoFrame`. Platform-specific `#ifdef _WIN32` blocks for COM init, BSTR handling, WideCharToMultiByte.

### momo-gpu

- **Feature flag**: `gpu` — enables CUDA via `cudarc`. Default OFF (CPU fallback). Propagates: `momo-app/gpu` → `momo-pipeline/gpu` → `momo-gpu/gpu`.
- **`GpuProcessor`** — `new()`, `process(&Frame, &OutputTransform, Resolution) -> Result<Frame>`. With `gpu` feature + CUDA available: uses GPU kernels. Otherwise: CPU fallback. Graceful degradation if CUDA init fails.
- **`cuda`** module (feature-gated) — `CudaProcessor`: loads pre-compiled PTX (from `build.rs`/nvcc), provides `crop_uyvy()`, `scale_uyvy()`, `flip_uyvy()` via cudarc. Uses `CudaContext` + `CudaStream` + `launch_builder` API.
- **`transform`** module — CPU implementations: `crop_uyvy()` (2-pixel aligned), `scale_uyvy_nearest()` (macro-pixel aware), `flip_uyvy()` (horizontal swaps Y0/Y1 within macro-pixel).
- **`is_cuda_available()`** — with `gpu` feature: attempts `CudaContext::new(0)`. Without: returns false.
- **CUDA kernels** (`kernels/`): `crop.cu` (per-pixel), `scale.cu` (per-macro-pixel nearest-neighbor), `flip.cu` (per-macro-pixel with Y0/Y1 swap for horizontal). Compiled to PTX by `build.rs` via nvcc.

### momo-uvc

- **Feature flag**: `uvc` — enables `nokhwa` camera capture. Default OFF. Propagates: `momo-app/uvc` → `momo-pipeline/uvc` → `momo-uvc/uvc`.
- **`UvcInput`** (feature-gated) — captures frames via nokhwa on dedicated OS thread, converts YUYV→UYVY, sends via crossbeam channel.
- **`convert`** module — `yuyv_to_uyvy()`: byte-swap conversion (always available).
- **`enumerate_devices()`** — with `uvc` feature: queries via `nokhwa::query()`. Without: returns empty Vec.

### momo-webrtc

- **Feature flag**: `webrtc` — enables `str0m` (WebRTC) + `openh264` (H.264 encoding). Default OFF. Propagates: `momo-app/webrtc` → `momo-web/webrtc` → `momo-webrtc/webrtc`. Signal types (`signal` module) and `convert` module are always available (no feature gate).
- **`WebRtcManager`** — creates sessions with a `SubscribeFn` callback to subscribe to pipeline raw frames. Held in `AppState` as `Arc<WebRtcManager>`.
- **`SessionHandle`** — handle for an active session: `signal_tx` (client→session), `signal_rx` (session→client), spawned tokio task.
- **`run_session()`** (feature-gated) — async function managing str0m `Rtc` instance, UDP socket, H.264 encoding via OpenH264, and frame delivery from pipeline broadcast channels.
- **`H264Encoder`** (feature-gated) — wraps OpenH264 encoder, converts NV12 frames to H.264 `EncodedPacket`s.
- **`signal`** module — `ClientMessage` enum (`Answer`, `IceCandidate`, `SubscribeTrack`, `UnsubscribeTrack`) and `ServerMessage` enum (`Offer`, `IceCandidate`, `TrackAdded`, `TrackRemoved`, `Error`). Both use `#[serde(tag = "type")]`.
- **`convert`** module — `uyvy_to_nv12()`: UYVY 4:2:2 → NV12 4:2:0 conversion for H.264 encoding.

### momo-pipeline

- **`Pipeline`** — fields: `state`, `config: Option<Config>`, `config_path: Option<PathBuf>`, `event_tx: broadcast::Sender<PipelineEvent>`, `preview_tx: broadcast::Sender<Vec<u8>>`, `output_preview_txs: HashMap<String, broadcast::Sender<Vec<u8>>>`, `raw_preview_tx: broadcast::Sender<Arc<Frame>>`, `raw_output_preview_txs: HashMap<String, broadcast::Sender<Arc<Frame>>>`, `running: Option<RunningState>`. Methods: `new()`, `state()`, `config()`, `subscribe()`, `subscribe_preview()`, `subscribe_output_preview(id)`, `subscribe_raw_preview()`, `subscribe_raw_output_preview(id)`, `set_config()`, `set_config_path()`, `start()`, `stop()`, `save_config()`, `load_config()`, `update_output()`, `outputs()`.
- **Per-output preview**: `start()` creates per-output broadcast channels. Frame loop encodes output previews throttled at `PreviewConfig.fps`, skipping encode when `receiver_count() == 0`. Channels cleared on `stop()`.
- **DeckLink output** (feature-gated): `start()` creates and starts `DeckLinkOutput` per enabled output. Frame loop calls `VideoOutput::send_frame()` after GPU transform. Outputs stopped gracefully on task completion.
- **`PipelineEvent`** — `StateChanged { state }`, `FpsUpdate { fps: f64 }`, `DeviceEvent { device, status }`, `ConfigChanged`, `Error { message }`. Serialized as tagged JSON.
- **`InputDriver`** — enum: `Mock(MockInput)`, `DeckLink(DeckLinkInput)` (feature-gated), `Uvc(UvcInput)` (feature-gated). Factory: `from_config(&InputSource)`.
- **`MockInput`** — generates UYVY color bars (SMPTE 8-bar pattern: White/Yellow/Cyan/Green/Magenta/Red/Blue/Black). Runs on dedicated OS thread at configured FPS. Stop via `Arc<AtomicBool>`.
- **`preview.rs`** — `uyvy_to_rgb()` (BT.601 coefficients), `nearest_neighbor_scale()`, `encode_preview()` (→ JPEG via `image::codecs::jpeg::JpegEncoder`), `uyvy_to_nv12()` (UYVY 4:2:2 → NV12 4:2:0 for WebRTC H.264).

### momo-web

- **`AppState`** — `pipeline: Arc<RwLock<Pipeline>>`, `webrtc_manager: Arc<WebRtcManager>` (feature-gated). Constructor: `new(pipeline)` — with `webrtc` feature, creates `WebRtcManager` with a `SubscribeFn` that bridges to `subscribe_raw_preview()` / `subscribe_raw_output_preview()`.
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
GET    /api/preview/output/{id}   → multipart/x-mixed-replace MJPEG stream | 404
WS     /ws/status                 → PipelineEvent JSON messages
WS     /ws/preview                → WebRTC signaling (feature-gated: webrtc)
```

## Tests (60 total)

**momo-core** (7): `config_serde_roundtrip`, `config_file_roundtrip`, `config_rejects_empty_outputs`, `config_rejects_duplicate_ids`, `config_rejects_zero_crop`, `config_defaults_applied`, `input_source_uvc_roundtrip`.

**momo-decklink** (5): `display_mode_roundtrip`, `pixel_format_roundtrip`, `unknown_bmd_display_mode`, `unknown_bmd_pixel_format`, `known_bmd_constants`.

**momo-gpu** (12): `crop_uyvy_basic`, `crop_uyvy_full_frame`, `crop_uyvy_out_of_bounds`, `scale_uyvy_nearest_half`, `scale_uyvy_nearest_same_size`, `flip_uyvy_noop`, `flip_uyvy_vertical`, `flip_uyvy_horizontal`, `process_crop_scale_flip`, `process_identity`, `process_with_crop_and_scale`, `process_with_flip`.

**momo-uvc** (2): `yuyv_to_uyvy_basic`, `yuyv_to_uyvy_multiple_macropixels`.

**momo-pipeline** (15): `color_bar_frame_size`, `color_bar_frame_small` (mock_input), `initial_state_is_stopped`, `set_config`, `update_output_transform`, `update_output_not_found`, `start_stop_lifecycle`, `start_without_config_fails`, `stop_when_stopped_fails`, `subscribe_preview` (waits 3s for JPEG frame), `subscribe_output_preview_lifecycle` (pipeline), `uyvy_to_rgb_known_values`, `uyvy_to_nv12_basic`, `nearest_neighbor_scale_halves`, `encode_preview_produces_jpeg` (preview).

**momo-webrtc** (8): `client_answer_roundtrip`, `client_subscribe_roundtrip`, `client_ice_candidate_roundtrip`, `server_offer_roundtrip`, `server_ice_candidate_roundtrip`, `server_error_roundtrip` (signal), `uyvy_to_nv12_basic`, `uyvy_to_nv12_4x4` (convert).

**momo-web** (11): `get_status_returns_stopped`, `get_config_returns_config`, `get_config_no_config_returns_400`, `put_config_sets_config`, `get_devices_returns_array`, `start_stop_pipeline`, `stop_when_stopped_returns_conflict`, `preview_output_returns_404_when_stopped`, `preview_output_returns_mjpeg_when_running`, `preview_input_returns_mjpeg_content_type`, `update_output_transform`.

## Frontend (SolidJS + Vite)

`frontend/` — SolidJS SPA with TypeScript. Uses `vite-plugin-singlefile` to produce a single HTML file for binary embedding. Dev proxy: `/api` → `:8080`, `/ws` → `ws://:8080`.

**Components**: `App` (root, state management, WebSocket, single source of truth for config), `StatusBar` (state badge, FPS, start/stop), `InputPanel` (source label, MJPEG preview when running, hosts CropOverlay), `CropOverlay` (MJPEG preview + draggable/resizable crop rects per output, uses ResizeObserver + `object-fit: contain` coordinate mapping), `CropRect` (draggable/resizable crop rectangle with 8 handles, only interactive when `selected`), `OutputList`/`OutputCard` (per-output crop/flip editing with Edit Crop → Apply/Cancel flow, live output preview thumbnail when pipeline running), `PreviewImage` (img tag), `WebRTCPreview` (video element for WebRTC MediaStream with MJPEG fallback), `FpsChart` (real-time FPS visualization via canvas), `Waveform` (video-driven waveform analyzer), `ConfigActions` (save/load buttons).

**Crop editing flow**: OutputCard "Edit Crop" → sets `editing` state + `selectedOutputId` → CropOverlay shows handles on selected rect → drag/resize calls `onCropChange` → App updates config signal (single source of truth) → OutputCard reads crop from `props.output.transform.crop` (no local crop state) → "Apply" sends to backend → "Cancel" reverts. Number inputs in OutputCard also call `onCropChange` to stay in sync with overlay.

**Coordinate utils** (`src/utils/coordinates.ts`): `inputToPreview()` / `previewToInput()` convert between input pixel coords and preview CSS coords. `DISPLAY_MODE_RESOLUTIONS` maps mode names to resolutions. `OUTPUT_COLORS` palette for per-output coloring.

**API client** (`src/api/client.ts`): `getStatus`, `getConfig`, `putConfig`, `updateOutput`, `saveConfig`, `loadConfig`, `startPipeline`, `stopPipeline`. All requests use `cache: 'no-store'` to prevent stale GET responses after mutations.

**WebRTC client** (`src/api/webrtc.ts`): `PreviewStream` class — manages `RTCPeerConnection` and signaling via WebSocket to `/ws/preview`. Handles `SubscribeTrack`/`UnsubscribeTrack` for input and per-output streams.

**WebSocket** (`src/api/websocket.ts`): auto-reconnect with 2s delay.

**SolidJS patterns**: `OutputList` uses `<Index>` (not `<For>`) to track outputs by array index, preventing component recreation on config refresh. SolidJS `on*` event handlers are evaluated once at mount — conditional logic must be inside the handler, not in the JSX attribute expression.

## CI (GitHub Actions)

**check** job (ubuntu): Node 22 → `npm install && npm run build` → `cargo build` → `cargo clippy -- -D warnings` → `cargo test` (no hardware features).

**build-linux** job (nvidia/cuda:12.6 container, needs check): Rust + Node → frontend build → `cargo clippy/test/build --release --features gpu,decklink` → upload `momo-linux-x86_64` artifact.

**build-windows** job (windows-latest, needs check): MSVC + CUDA 12.6 + Node → frontend build → `cargo build --release --target x86_64-pc-windows-msvc --features gpu,decklink` → upload `momo-windows-x86_64` artifact.

Frontend is built BEFORE cargo so that `build.rs` can embed `frontend/dist/index.html` into the binary.

## Key Constraints

- Live/broadcast use — low latency is critical, no allocations in hot path
- DeckLink SDK is C++; access via `cxx` crate FFI bindings (not bindgen). Linux: dlopen via DeckLinkAPIDispatch. Windows: COM via midl-generated headers
- Must handle DeckLink hot-unplug gracefully (no crash, status reflected in UI)
- Cross-platform: Ubuntu and Windows
- GPU: CUDA via `cudarc`, targeting GTX 1080+
- DeckLink/CUDA/UVC are feature-gated — all tests must pass without hardware (default build). With `--features gpu,decklink`, tests pass with graceful fallback (DeckLink output warns if no hardware; CUDA uses GPU if available, CPU otherwise)
- When `--config` file doesn't exist, a default Mock input config is generated automatically
- Config uses apply-button paradigm — changes are explicit, not auto-applied

## Language

- Code, comments, and commit messages: English
- UI text and documentation: Japanese is acceptable
