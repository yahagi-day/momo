// Scale kernel — nearest-neighbor scaling of a UYVY frame.
// Works at macro-pixel (4 bytes = 2 pixels) granularity to preserve chroma pairing.

extern "C" __global__ void scale_uyvy(
    const unsigned char* __restrict__ src,
    unsigned char* __restrict__ dst,
    int src_width,
    int src_height,
    int dst_width,
    int dst_height
) {
    // Each thread handles one macro-pixel (2 pixels) in the destination
    int mx = blockIdx.x * blockDim.x + threadIdx.x;
    int y  = blockIdx.y * blockDim.y + threadIdx.y;

    int dst_macro_w = dst_width / 2;
    int src_macro_w = src_width / 2;

    if (mx < dst_macro_w && y < dst_height) {
        int sy  = (int)((long long)y  * src_height / dst_height);
        int smx = (int)((long long)mx * src_macro_w / dst_macro_w);

        int src_offset = sy * src_width * 2 + smx * 4;
        int dst_offset = y  * dst_width * 2 + mx  * 4;

        dst[dst_offset]     = src[src_offset];     // U
        dst[dst_offset + 1] = src[src_offset + 1]; // Y0
        dst[dst_offset + 2] = src[src_offset + 2]; // V
        dst[dst_offset + 3] = src[src_offset + 3]; // Y1
    }
}
