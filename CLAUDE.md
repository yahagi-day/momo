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

**Threading model**: Mock/hardware input runs on a dedicated OS thread. A bridge task forwards frames from `crossbeam-channel` to `tokio::sync::mpsc`. Preview encoding and FPS tracking run as tokio tasks. Web/API runs on the tokio async runtime.

**Frame flow**: Input(CPU) → crossbeam channel → bridge → preview encode (UYVY→RGB→scale→JPEG) → `broadcast::Sender<Vec<u8>>` → MJPEG endpoint.

**UI embedding**: `momo-web/build.rs` embeds the UI HTML into the binary at compile time via `include_str!`. If `frontend/dist/index.html` exists (from Vite + `vite-plugin-singlefile` build), it uses that; otherwise embeds `src/fallback.html`. The binary is fully self-contained.

**Pipeline lifecycle**: `Pipeline` is held in `Arc<RwLock<Pipeline>>` via `AppState`. `start()` creates InputDriver → spawns input thread → bridge task → preview task. `stop()` sets `AtomicBool` stop flag and aborts tasks.

## Key Types

- **`momo_core::Config`** — top-level config with `load()`/`save()`/`validate()`. Serialized as tagged JSON (`InputSource` uses `#[serde(tag = "type")]`).
- **`momo_core::Frame`** — CPU-side video frame (data + resolution + format + timestamp).
- **`momo_core::InputSource`** — `DeckLink`, `Uvc`, or `Mock { width, height, fps }`.
- **`momo_pipeline::Pipeline`** — orchestrates input→preview→outputs, tracks `PipelineState`, owns `broadcast::Sender` for events and preview frames.
- **`momo_pipeline::PipelineEvent`** — `StateChanged`, `FpsUpdate`, `ConfigChanged`, `Error` (serialized to JSON, sent over WebSocket).
- **`momo_pipeline::InputDriver`** — enum dispatching to `MockInput` (future: DeckLink, Uvc). Created via `from_config()`.
- **`momo_web::AppState`** — `Arc<RwLock<Pipeline>>`, passed to all axum handlers.

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
