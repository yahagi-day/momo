/// Convert YUYV (YUY2) to UYVY by swapping byte pairs.
///
/// YUYV: Y0 U0 Y1 V0
/// UYVY: U0 Y0 V0 Y1
pub fn yuyv_to_uyvy(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; data.len()];
    for (src, dst) in data.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
        dst[0] = src[1]; // U
        dst[1] = src[0]; // Y0
        dst[2] = src[3]; // V
        dst[3] = src[2]; // Y1
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuyv_to_uyvy_basic() {
        // YUYV: Y0=100, U=50, Y1=110, V=60
        let yuyv = vec![100, 50, 110, 60];
        let uyvy = yuyv_to_uyvy(&yuyv);
        // UYVY: U=50, Y0=100, V=60, Y1=110
        assert_eq!(uyvy, vec![50, 100, 60, 110]);
    }

    #[test]
    fn yuyv_to_uyvy_multiple_macropixels() {
        let yuyv = vec![
            100, 50, 110, 60, // macro-pixel 0
            200, 150, 210, 160, // macro-pixel 1
        ];
        let uyvy = yuyv_to_uyvy(&yuyv);
        assert_eq!(
            uyvy,
            vec![
                50, 100, 60, 110, // macro-pixel 0 swapped
                150, 200, 160, 210, // macro-pixel 1 swapped
            ]
        );
    }
}
