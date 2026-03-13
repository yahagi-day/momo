//! GPU processing via CUDA.
//!
//! Provides:
//! - CUDA context and buffer pool management
//! - Crop, scale, flip kernels (loaded as PTX)
//! - Per-output CUDA stream processing

/// Placeholder for GPU context initialization.
pub fn is_cuda_available() -> bool {
    tracing::warn!("CUDA availability check not yet implemented");
    false
}
