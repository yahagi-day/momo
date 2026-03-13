// Scale kernel — bilinear scaling of a UYVY frame.
// Stub: to be implemented in Phase 2.

extern "C" __global__ void scale_uyvy(
    const unsigned char* __restrict__ src,
    unsigned char* __restrict__ dst,
    int src_width,
    int src_height,
    int dst_width,
    int dst_height
) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;

    if (x < dst_width && y < dst_height) {
        // Nearest-neighbor placeholder — bilinear to follow
        float src_x = (float)x * src_width / dst_width;
        float src_y = (float)y * src_height / dst_height;
        int sx = (int)src_x;
        int sy = (int)src_y;
        if (sx >= src_width) sx = src_width - 1;
        if (sy >= src_height) sy = src_height - 1;

        int src_offset = (sy * src_width + sx) * 2;
        int dst_offset = (y * dst_width + x) * 2;
        dst[dst_offset]     = src[src_offset];
        dst[dst_offset + 1] = src[src_offset + 1];
    }
}
