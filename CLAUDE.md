# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**momo** is a Rust-based live video splitter/router. It captures a single video input (DeckLink or UVC), applies per-output GPU transformations (crop → scale → flip), and outputs to multiple DeckLink devices simultaneously. A web UI provides configuration, preview, and monitoring.

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

## Architecture

Cargo workspace with 7 crates. Dependency flow is strictly one-directional:

```
momo-core          shared types (Frame, Config, Error, PixelFormat, DisplayMode, etc.)
  ↑
  ├── momo-decklink   DeckLink FFI — VideoInput/VideoOutput traits, device enumeration
  ├── momo-uvc        UVC camera input (v4l2 / MediaFoundation)
  ├── momo-gpu        CUDA kernels (crop/scale/flip as PTX) + buffer management
  │
  ├── momo-pipeline   frame routing: input → GPU → N outputs (uses decklink, uvc, gpu)
  │     ↑
  │     └── momo-web  axum REST API + WebSocket + MJPEG preview
  │           ↑
  └───────── momo-app  binary entry point (clap CLI, tracing init, wires everything)
```

**Threading model**: Video capture/processing runs on dedicated OS threads for latency stability. Web/API runs on tokio async runtime. `crossbeam-channel` bridges the two worlds.

**Frame flow**: Input(CPU) → GPU upload → parallel CUDA streams (one per output: crop→scale→flip) → D2H → DeckLink outputs. All outputs share the same GPU source buffer.

## Key Types

- **`momo_core::Config`** — top-level config with `load()`/`save()`/`validate()`. Serialized as tagged JSON (`InputSource` uses `#[serde(tag = "type")]`).
- **`momo_core::Frame`** — CPU-side video frame (data + resolution + format + timestamp).
- **`momo_decklink::VideoInput` / `VideoOutput`** — traits for capture and playout (Send-bounded for threading).
- **`momo_pipeline::Pipeline`** — orchestrates input→GPU→outputs, tracks `PipelineState`.
- **`momo_core::OutputTransform`** — per-output transform chain: `crop: Option<CropRegion>` → scale (implicit to output format) → `flip: FlipOptions`.

## Key Constraints

- Live/broadcast use — low latency is critical, no allocations in hot path
- DeckLink SDK is C++; access via `cxx` crate FFI bindings (not bindgen)
- Must handle DeckLink hot-unplug gracefully (no crash, status reflected in UI)
- Cross-platform: Ubuntu and Windows
- GPU: CUDA via `cudarc`, targeting GTX 1080+
- Rotation (90°/180°/270°) is planned for future; transforms are designed to be extensible
- Config uses apply-button paradigm — changes are explicit, not auto-applied

## Language

- Code, comments, and commit messages: English
- UI text and documentation: Japanese is acceptable
