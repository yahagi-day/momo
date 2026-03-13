use image::codecs::jpeg::JpegEncoder;
use image::ImageEncoder;
use momo_core::config::PreviewConfig;
use momo_core::error::{Error, Result};
use momo_core::frame::Frame;

/// Convert UYVY pixel data to RGB.
pub fn uyvy_to_rgb(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut rgb = Vec::with_capacity(pixel_count * 3);

    for chunk in data.chunks_exact(4) {
        let u = chunk[0] as f32 - 128.0;
        let y0 = chunk[1] as f32 - 16.0;
        let v = chunk[2] as f32 - 128.0;
        let y1 = chunk[3] as f32 - 16.0;

        for y in [y0, y1] {
            let r = (1.164 * y + 1.596 * v).clamp(0.0, 255.0) as u8;
            let g = (1.164 * y - 0.392 * u - 0.813 * v).clamp(0.0, 255.0) as u8;
            let b = (1.164 * y + 2.017 * u).clamp(0.0, 255.0) as u8;
            rgb.push(r);
            rgb.push(g);
            rgb.push(b);
        }
    }

    rgb
}

/// Nearest-neighbor scale RGB data.
pub fn nearest_neighbor_scale(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dst_w: u32,
    dst_h: u32,
) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_w * dst_h * 3) as usize];

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx as u64 * src_w as u64 / dst_w as u64) as usize;
            let sy = (dy as u64 * src_h as u64 / dst_h as u64) as usize;
            let si = (sy * src_w as usize + sx) * 3;
            let di = (dy * dst_w + dx) as usize * 3;
            dst[di..di + 3].copy_from_slice(&src[si..si + 3]);
        }
    }

    dst
}

/// Encode a Frame as a JPEG preview image.
pub fn encode_preview(frame: &Frame, config: &PreviewConfig) -> Result<Vec<u8>> {
    let rgb = uyvy_to_rgb(&frame.data, frame.resolution.width, frame.resolution.height);
    let scaled = nearest_neighbor_scale(
        &rgb,
        frame.resolution.width,
        frame.resolution.height,
        config.width,
        config.height,
    );

    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, config.jpeg_quality);
    encoder
        .write_image(
            &scaled,
            config.width,
            config.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| Error::Pipeline(format!("JPEG encode error: {e}")))?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use momo_core::types::{PixelFormat, Resolution};

    #[test]
    fn uyvy_to_rgb_known_values() {
        // White pixel in UYVY: U=128, Y=235, V=128, Y=235
        let data = vec![128, 235, 128, 235];
        let rgb = uyvy_to_rgb(&data, 2, 1);
        assert_eq!(rgb.len(), 6);
        // Should produce near-white (Y=235 is full-range white)
        assert!(rgb[0] > 200); // R
        assert!(rgb[1] > 200); // G
        assert!(rgb[2] > 200); // B
    }

    #[test]
    fn nearest_neighbor_scale_halves() {
        // 4x2 RGB → 2x1
        let src = vec![
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0, // row 0
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 1
        ];
        let dst = nearest_neighbor_scale(&src, 4, 2, 2, 1);
        assert_eq!(dst.len(), 6);
        assert_eq!(&dst[0..3], &[255, 0, 0]); // sampled from (0,0)
        assert_eq!(&dst[3..6], &[0, 0, 255]); // sampled from (2,0)
    }

    #[test]
    fn encode_preview_produces_jpeg() {
        let width = 320u32;
        let height = 240u32;
        let data = vec![128u8; (width * height * 2) as usize];
        let frame = Frame {
            data,
            resolution: Resolution { width, height },
            format: PixelFormat::Uyvy,
            timestamp_ns: 0,
            sequence: 0,
        };
        let config = PreviewConfig::default();
        let jpeg = encode_preview(&frame, &config).unwrap();
        // JPEG magic bytes
        assert!(jpeg.len() > 2);
        assert_eq!(jpeg[0], 0xFF);
        assert_eq!(jpeg[1], 0xD8);
    }
}
