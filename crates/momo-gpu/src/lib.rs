//! GPU processing via CUDA with CPU fallback.
//!
//! Provides frame transformation: crop → scale → flip.
//!
//! When the `gpu` feature is enabled, CUDA kernels are used.
//! Otherwise, CPU fallback implementations are provided.

#[cfg(feature = "gpu")]
pub mod cuda;
pub mod processor;
pub mod transform;

pub use processor::GpuProcessor;

/// Check if CUDA is available on this system.
pub fn is_cuda_available() -> bool {
    #[cfg(feature = "gpu")]
    {
        std::panic::catch_unwind(|| cudarc::driver::CudaContext::new(0).is_ok()).unwrap_or(false)
    }
    #[cfg(not(feature = "gpu"))]
    {
        false
    }
}
