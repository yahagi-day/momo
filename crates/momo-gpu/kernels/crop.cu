// Crop kernel — extracts a rectangular region from the source frame.
// Will be compiled to PTX by build.rs and loaded at runtime via cudarc.

extern "C" __global__ void crop_uyvy(
    const unsigned char* __restrict__ src,
    unsigned char* __restrict__ dst,
    int src_width,
    int src_height,
    int crop_x,
    int crop_y,
    int crop_width,
    int crop_height
) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;

    if (x < crop_width && y < crop_height) {
        // UYVY: 2 bytes per pixel, but pixels are paired (macro-pixel = 4 bytes for 2 pixels)
        int src_offset = ((crop_y + y) * src_width + (crop_x + x)) * 2;
        int dst_offset = (y * crop_width + x) * 2;
        dst[dst_offset]     = src[src_offset];
        dst[dst_offset + 1] = src[src_offset + 1];
    }
}
