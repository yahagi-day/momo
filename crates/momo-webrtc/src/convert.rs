//! Color space conversion utilities for WebRTC encoding.

/// Convert UYVY 4:2:2 pixel data to NV12 4:2:0.
///
/// NV12 layout: Y plane (width*height bytes), followed by interleaved UV plane
/// (width*height/2 bytes, subsampled vertically by 2).
pub fn uyvy_to_nv12(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let y_size = w * h;
    let uv_size = w * (h / 2);
    let mut nv12 = vec![0u8; y_size + uv_size];

    let (y_plane, uv_plane) = nv12.split_at_mut(y_size);
    let uyvy_stride = w * 2;

    for row in 0..h {
        let uyvy_row = &data[row * uyvy_stride..(row + 1) * uyvy_stride];
        let y_row = &mut y_plane[row * w..(row + 1) * w];

        for x in (0..w).step_by(2) {
            let base = x * 2;
            y_row[x] = uyvy_row[base + 1];     // Y0
            y_row[x + 1] = uyvy_row[base + 3]; // Y1
        }

        if row % 2 == 0 {
            let uv_row = &mut uv_plane[(row / 2) * w..(row / 2 + 1) * w];
            for x in (0..w).step_by(2) {
                let base = x * 2;
                uv_row[x] = uyvy_row[base];         // U
                uv_row[x + 1] = uyvy_row[base + 2]; // V
            }
        }
    }

    nv12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uyvy_to_nv12_basic() {
        let data = vec![
            128, 200, 128, 210, // row 0: U=128, Y0=200, V=128, Y1=210
            100, 150, 200, 160, // row 1: U=100, Y0=150, V=200, Y1=160
        ];
        let nv12 = uyvy_to_nv12(&data, 2, 2);
        assert_eq!(nv12.len(), 6);
        assert_eq!(nv12[0], 200);
        assert_eq!(nv12[1], 210);
        assert_eq!(nv12[2], 150);
        assert_eq!(nv12[3], 160);
        assert_eq!(nv12[4], 128);
        assert_eq!(nv12[5], 128);
    }

    #[test]
    fn uyvy_to_nv12_4x4() {
        let w = 4u32;
        let h = 4u32;
        let data = vec![128u8; (w * h * 2) as usize];
        let nv12 = uyvy_to_nv12(&data, w, h);
        let expected_size = (w * h + w * h / 2) as usize;
        assert_eq!(nv12.len(), expected_size);
        for &y in &nv12[..(w * h) as usize] {
            assert_eq!(y, 128);
        }
    }
}
