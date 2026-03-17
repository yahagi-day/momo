// Flip kernel — horizontal and/or vertical flip of a UYVY frame.
// Works at macro-pixel (4 bytes = 2 pixels) granularity.
// Horizontal flip reverses macro-pixel order and swaps Y0/Y1 within each macro-pixel.

extern "C" __global__ void flip_uyvy(
    const unsigned char* __restrict__ src,
    unsigned char* __restrict__ dst,
    int width,
    int height,
    int flip_h,
    int flip_v
) {
    // Each thread handles one macro-pixel (2 pixels)
    int mx = blockIdx.x * blockDim.x + threadIdx.x;
    int y  = blockIdx.y * blockDim.y + threadIdx.y;

    int macro_w = width / 2;

    if (mx < macro_w && y < height) {
        int src_y  = flip_v ? (height - 1 - y) : y;
        int src_mx = flip_h ? (macro_w - 1 - mx) : mx;

        int src_offset = src_y * width * 2 + src_mx * 4;
        int dst_offset = y     * width * 2 + mx     * 4;

        if (flip_h) {
            // Reverse macro-pixel order, swap Y0 and Y1 positions
            dst[dst_offset]     = src[src_offset];     // U
            dst[dst_offset + 1] = src[src_offset + 3]; // Y1 -> Y0
            dst[dst_offset + 2] = src[src_offset + 2]; // V
            dst[dst_offset + 3] = src[src_offset + 1]; // Y0 -> Y1
        } else {
            // Vertical only: copy macro-pixel as-is
            dst[dst_offset]     = src[src_offset];
            dst[dst_offset + 1] = src[src_offset + 1];
            dst[dst_offset + 2] = src[src_offset + 2];
            dst[dst_offset + 3] = src[src_offset + 3];
        }
    }
}
