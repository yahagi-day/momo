use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::Sender;
use momo_core::frame::Frame;
use momo_core::types::{DisplayMode, PixelFormat, Resolution};

use crate::conversions::{display_mode_to_bmd, pixel_format_to_bmd};
use crate::ffi::decklink_ffi;

pub struct DeckLinkInput {
    device_index: u32,
    display_mode: DisplayMode,
    pixel_format: PixelFormat,
}

impl DeckLinkInput {
    pub fn new(device_index: u32, display_mode: DisplayMode, pixel_format: PixelFormat) -> Self {
        Self {
            device_index,
            display_mode,
            pixel_format,
        }
    }

    pub fn start(
        self,
        tx: Sender<Frame>,
        stop_flag: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        let bmd_mode = display_mode_to_bmd(self.display_mode);
        let bmd_fmt = pixel_format_to_bmd(self.pixel_format);
        let resolution = self.display_mode.resolution();
        let pixel_format = self.pixel_format;
        let device_index = self.device_index;

        std::thread::Builder::new()
            .name("decklink-input".into())
            .spawn(move || {
                tracing::info!(
                    "DeckLink input starting: device={}, mode=0x{:08X}, fmt=0x{:08X}",
                    device_index,
                    bmd_mode,
                    bmd_fmt,
                );

                let system = decklink_ffi::create_system();
                if system.is_null() || !system.is_api_present() {
                    tracing::error!("DeckLink API not available");
                    return;
                }

                let mut capture =
                    decklink_ffi::create_capture(&system, device_index, bmd_mode, bmd_fmt);
                if capture.is_null() {
                    tracing::error!("failed to create DeckLink capture for device {}", device_index);
                    return;
                }

                if !capture.as_mut().unwrap().start() {
                    tracing::error!("failed to start DeckLink capture");
                    return;
                }

                tracing::info!("DeckLink input running");
                let start = Instant::now();
                let mut sequence = 0u64;

                while !stop_flag.load(Ordering::Relaxed) {
                    let data = capture.as_mut().unwrap().get_frame(100);
                    if data.is_empty() {
                        continue;
                    }

                    let frame = Frame {
                        data: std::sync::Arc::new(data),
                        resolution: Resolution {
                            width: resolution.width,
                            height: resolution.height,
                        },
                        format: pixel_format,
                        timestamp_ns: start.elapsed().as_nanos() as u64,
                        sequence,
                    };

                    if tx.send(frame).is_err() {
                        break;
                    }
                    sequence += 1;
                }

                capture.as_mut().unwrap().stop();
                tracing::info!("DeckLink input stopped");
            })
            .expect("failed to spawn DeckLink input thread")
    }
}
