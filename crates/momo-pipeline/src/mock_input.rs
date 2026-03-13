use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use momo_core::frame::Frame;
use momo_core::types::{PixelFormat, Resolution};

pub struct MockInput {
    width: u32,
    height: u32,
    fps: u32,
}

// UYVY color bar values (Y, U, V) for standard SMPTE bars
const COLOR_BARS: [(u8, u8, u8); 8] = [
    (235, 128, 128), // White
    (210, 16, 146),  // Yellow
    (170, 166, 16),  // Cyan
    (145, 54, 34),   // Green
    (107, 202, 222), // Magenta
    (82, 90, 240),   // Red
    (41, 240, 110),  // Blue
    (16, 128, 128),  // Black
];

fn generate_color_bars(width: u32, height: u32) -> Vec<u8> {
    let bar_width = width / 8;
    let row_bytes = (width * 2) as usize;
    let mut data = vec![0u8; row_bytes * height as usize];

    for y in 0..height {
        for x in (0..width).step_by(2) {
            let bar_idx = ((x / bar_width) as usize).min(7);
            let (yy, u, v) = COLOR_BARS[bar_idx];
            let offset = (y * width * 2 + x * 2) as usize;
            data[offset] = u;
            data[offset + 1] = yy;
            data[offset + 2] = v;
            data[offset + 3] = yy;
        }
    }

    data
}

impl MockInput {
    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        Self { width, height, fps }
    }

    pub fn start(
        self,
        tx: Sender<Frame>,
        stop_flag: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("mock-input".into())
            .spawn(move || {
                tracing::info!(
                    "mock input started: {}x{} @ {}fps",
                    self.width,
                    self.height,
                    self.fps
                );
                let frame_data = generate_color_bars(self.width, self.height);
                let interval = Duration::from_secs_f64(1.0 / self.fps as f64);
                let mut sequence = 0u64;
                let start = Instant::now();

                while !stop_flag.load(Ordering::Relaxed) {
                    let frame = Frame {
                        data: frame_data.clone(),
                        resolution: Resolution {
                            width: self.width,
                            height: self.height,
                        },
                        format: PixelFormat::Uyvy,
                        timestamp_ns: start.elapsed().as_nanos() as u64,
                        sequence,
                    };

                    if tx.send(frame).is_err() {
                        break;
                    }

                    sequence += 1;
                    let next = start + interval * sequence as u32;
                    let now = Instant::now();
                    if next > now {
                        std::thread::sleep(next - now);
                    }
                }

                tracing::info!("mock input stopped");
            })
            .expect("failed to spawn mock input thread")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_bar_frame_size() {
        let data = generate_color_bars(1920, 1080);
        assert_eq!(data.len(), 1920 * 1080 * 2);
    }

    #[test]
    fn color_bar_frame_small() {
        let data = generate_color_bars(320, 240);
        assert_eq!(data.len(), 320 * 240 * 2);
    }
}
