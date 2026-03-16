use momo_core::error::{Error, Result};
use momo_core::types::{CropRegion, Resolution};

/// Crop UYVY frame data to the specified region.
///
/// UYVY has 2-pixel horizontal units (4 bytes per 2 pixels),
/// so x and width are aligned to 2-pixel boundaries.
pub fn crop_uyvy(
    data: &[u8],
    src_resolution: Resolution,
    crop: &CropRegion,
) -> Result<Vec<u8>> {
    // Align x and width to 2-pixel boundary
    let x = crop.x & !1;
    let width = (crop.width + (crop.x - x)) & !1;
    let y = crop.y;
    let height = crop.height;

    if x + width > src_resolution.width || y + height > src_resolution.height {
        return Err(Error::Gpu(format!(
            "crop region ({x},{y} {width}x{height}) exceeds source {}x{}",
            src_resolution.width, src_resolution.height
        )));
    }

    if width == 0 || height == 0 {
        return Err(Error::Gpu("crop region has zero dimension".into()));
    }

    let src_stride = src_resolution.width as usize * 2;
    let dst_stride = width as usize * 2;
    let mut out = vec![0u8; dst_stride * height as usize];

    let x_byte_offset = x as usize * 2;
    for row in 0..height as usize {
        let src_start = (y as usize + row) * src_stride + x_byte_offset;
        let dst_start = row * dst_stride;
        out[dst_start..dst_start + dst_stride]
            .copy_from_slice(&data[src_start..src_start + dst_stride]);
    }

    Ok(out)
}

/// Nearest-neighbor scale UYVY data to target resolution.
///
/// Both source and destination widths must be even.
pub fn scale_uyvy_nearest(
    data: &[u8],
    src_resolution: Resolution,
    dst_resolution: Resolution,
) -> Vec<u8> {
    let src_w = src_resolution.width as usize;
    let dst_w = dst_resolution.width as usize;
    let dst_h = dst_resolution.height as usize;

    // Work in 2-pixel UYVY macro-pixel units
    let src_macro_w = src_w / 2;
    let dst_macro_w = dst_w / 2;
    let src_stride = src_w * 2;

    let mut out = vec![0u8; dst_w * 2 * dst_h];

    for dy in 0..dst_h {
        let sy = dy as u64 * src_resolution.height as u64 / dst_resolution.height as u64;
        let sy = sy as usize;
        let src_row = &data[sy * src_stride..(sy + 1) * src_stride];

        for dx_macro in 0..dst_macro_w {
            let sx_macro =
                dx_macro as u64 * src_macro_w as u64 / dst_macro_w as u64;
            let sx_macro = sx_macro as usize;

            let si = sx_macro * 4;
            let di = dx_macro * 4 + dy * dst_w * 2;

            out[di] = src_row[si];         // U
            out[di + 1] = src_row[si + 1]; // Y0
            out[di + 2] = src_row[si + 2]; // V
            out[di + 3] = src_row[si + 3]; // Y1
        }
    }

    out
}

/// Flip UYVY data horizontally and/or vertically.
pub fn flip_uyvy(
    data: &[u8],
    resolution: Resolution,
    horizontal: bool,
    vertical: bool,
) -> Vec<u8> {
    if !horizontal && !vertical {
        return data.to_vec();
    }

    let w = resolution.width as usize;
    let h = resolution.height as usize;
    let stride = w * 2;
    let macro_w = w / 2; // Number of UYVY macro-pixels per row
    let mut out = vec![0u8; data.len()];

    for row in 0..h {
        let dst_row = if vertical { h - 1 - row } else { row };

        if horizontal {
            // Reverse macro-pixel order within the row, also swap U and V
            let src_row_start = row * stride;
            let dst_row_start = dst_row * stride;

            for mx in 0..macro_w {
                let src_mx = mx * 4 + src_row_start;
                let dst_mx = (macro_w - 1 - mx) * 4 + dst_row_start;

                // When flipping horizontally, Y0 and Y1 swap positions,
                // and we swap U/V to maintain correct chroma
                out[dst_mx] = data[src_mx];         // U stays U
                out[dst_mx + 1] = data[src_mx + 3]; // Y1 → Y0
                out[dst_mx + 2] = data[src_mx + 2]; // V stays V
                out[dst_mx + 3] = data[src_mx + 1]; // Y0 → Y1
            }
        } else {
            // Vertical only: copy row as-is to flipped row position
            let src_start = row * stride;
            let dst_start = dst_row * stride;
            out[dst_start..dst_start + stride]
                .copy_from_slice(&data[src_start..src_start + stride]);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use momo_core::types::Resolution;

    fn make_uyvy_frame(w: u32, h: u32) -> Vec<u8> {
        // Fill with identifiable pattern: each macro-pixel gets unique values
        let macro_w = w as usize / 2;
        let mut data = vec![0u8; w as usize * 2 * h as usize];
        for row in 0..h as usize {
            for mx in 0..macro_w {
                let offset = row * w as usize * 2 + mx * 4;
                let val = ((row * macro_w + mx) & 0xFF) as u8;
                data[offset] = val;         // U
                data[offset + 1] = val + 1; // Y0
                data[offset + 2] = val + 2; // V
                data[offset + 3] = val + 3; // Y1
            }
        }
        data
    }

    #[test]
    fn crop_uyvy_basic() {
        let w = 8u32;
        let h = 4u32;
        let data = make_uyvy_frame(w, h);
        let res = Resolution { width: w, height: h };
        let crop = CropRegion { x: 2, y: 1, width: 4, height: 2 };

        let cropped = crop_uyvy(&data, res, &crop).unwrap();
        // 4 pixels wide × 2 rows = 4*2*2 = 16 bytes
        assert_eq!(cropped.len(), 16);
    }

    #[test]
    fn crop_uyvy_full_frame() {
        let w = 8u32;
        let h = 4u32;
        let data = make_uyvy_frame(w, h);
        let res = Resolution { width: w, height: h };
        let crop = CropRegion { x: 0, y: 0, width: w, height: h };

        let cropped = crop_uyvy(&data, res, &crop).unwrap();
        assert_eq!(cropped, data);
    }

    #[test]
    fn crop_uyvy_out_of_bounds() {
        let w = 8u32;
        let h = 4u32;
        let data = make_uyvy_frame(w, h);
        let res = Resolution { width: w, height: h };
        let crop = CropRegion { x: 6, y: 0, width: 4, height: 4 };

        assert!(crop_uyvy(&data, res, &crop).is_err());
    }

    #[test]
    fn scale_uyvy_nearest_half() {
        let w = 8u32;
        let h = 4u32;
        let data = make_uyvy_frame(w, h);
        let src_res = Resolution { width: w, height: h };
        let dst_res = Resolution { width: 4, height: 2 };

        let scaled = scale_uyvy_nearest(&data, src_res, dst_res);
        assert_eq!(scaled.len(), 4 * 2 * 2); // 4 pixels × 2 rows × 2 bytes/pixel
    }

    #[test]
    fn scale_uyvy_nearest_same_size() {
        let w = 4u32;
        let h = 2u32;
        let data = make_uyvy_frame(w, h);
        let res = Resolution { width: w, height: h };

        let scaled = scale_uyvy_nearest(&data, res, res);
        assert_eq!(scaled, data);
    }

    #[test]
    fn flip_uyvy_noop() {
        let w = 4u32;
        let h = 2u32;
        let data = make_uyvy_frame(w, h);
        let res = Resolution { width: w, height: h };

        let result = flip_uyvy(&data, res, false, false);
        assert_eq!(result, data);
    }

    #[test]
    fn flip_uyvy_vertical() {
        let w = 4u32;
        let h = 2u32;
        let data = make_uyvy_frame(w, h);
        let res = Resolution { width: w, height: h };
        let stride = w as usize * 2;

        let flipped = flip_uyvy(&data, res, false, true);
        // First row of flipped should be last row of source
        assert_eq!(&flipped[..stride], &data[stride..]);
        assert_eq!(&flipped[stride..], &data[..stride]);
    }

    #[test]
    fn flip_uyvy_horizontal() {
        // 4 pixels (2 macro-pixels), 1 row
        let data: Vec<u8> = vec![
            10, 11, 12, 13, // macro-pixel 0: U=10, Y0=11, V=12, Y1=13
            20, 21, 22, 23, // macro-pixel 1: U=20, Y0=21, V=22, Y1=23
        ];
        let res = Resolution { width: 4, height: 1 };

        let flipped = flip_uyvy(&data, res, true, false);
        // macro-pixel 1 → position 0, with Y0/Y1 swapped
        assert_eq!(flipped[0], 20); // U from macro 1
        assert_eq!(flipped[1], 23); // Y1 from macro 1 → Y0
        assert_eq!(flipped[2], 22); // V from macro 1
        assert_eq!(flipped[3], 21); // Y0 from macro 1 → Y1
        // macro-pixel 0 → position 1, with Y0/Y1 swapped
        assert_eq!(flipped[4], 10); // U from macro 0
        assert_eq!(flipped[5], 13); // Y1 from macro 0 → Y0
        assert_eq!(flipped[6], 12); // V from macro 0
        assert_eq!(flipped[7], 11); // Y0 from macro 0 → Y1
    }

    #[test]
    fn process_crop_scale_flip() {
        // Integration: crop 4x2 from 8x4, scale to 4x2, flip vertical
        let w = 8u32;
        let h = 4u32;
        let data = make_uyvy_frame(w, h);
        let src_res = Resolution { width: w, height: h };

        let crop = CropRegion { x: 0, y: 0, width: 4, height: 4 };
        let cropped = crop_uyvy(&data, src_res, &crop).unwrap();
        let crop_res = Resolution { width: 4, height: 4 };

        let dst_res = Resolution { width: 4, height: 2 };
        let scaled = scale_uyvy_nearest(&cropped, crop_res, dst_res);

        let flipped = flip_uyvy(&scaled, dst_res, false, true);
        assert_eq!(flipped.len(), 4 * 2 * 2);
    }
}
