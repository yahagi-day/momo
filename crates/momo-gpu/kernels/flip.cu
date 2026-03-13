// Flip kernel — horizontal and/or vertical flip of a UYVY frame.
// Stub: to be implemented in Phase 2.

extern "C" __global__ void flip_uyvy(
    const unsigned char* __restrict__ src,
    unsigned char* __restrict__ dst,
    int width,
    int height,
    int flip_h,
    int flip_v
) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;

    if (x < width && y < height) {
        int src_x = flip_h ? (width - 1 - x) : x;
        int src_y = flip_v ? (height - 1 - y) : y;

        int src_offset = (src_y * width + src_x) * 2;
        int dst_offset = (y * width + x) * 2;
        dst[dst_offset]     = src[src_offset];
        dst[dst_offset + 1] = src[src_offset + 1];
    }
}
