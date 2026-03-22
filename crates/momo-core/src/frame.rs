use std::sync::Arc;

use crate::types::{PixelFormat, Resolution};

/// A video frame with CPU-side pixel data.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Pixel data in the specified format.
    pub data: Arc<Vec<u8>>,
    /// Frame resolution.
    pub resolution: Resolution,
    /// Pixel format.
    pub format: PixelFormat,
    /// Monotonic timestamp in nanoseconds.
    pub timestamp_ns: u64,
    /// Frame sequence number from the source.
    pub sequence: u64,
}

impl Frame {
    /// Expected byte size for the given resolution and format.
    pub fn expected_size(resolution: Resolution, format: PixelFormat) -> usize {
        let pixels = resolution.width as usize * resolution.height as usize;
        match format {
            PixelFormat::Uyvy => pixels * 2,
            PixelFormat::Bgra => pixels * 4,
            PixelFormat::V210 => {
                // v210: 128 bytes per 48 pixels (6 pixels per 16 bytes)
                let row_pixels = resolution.width as usize;
                let bytes_per_row = row_pixels.div_ceil(48) * 128;
                bytes_per_row * resolution.height as usize
            }
        }
    }
}
