//! GPU processing via CUDA with CPU fallback.
//!
//! Provides frame transformation: crop → scale → flip.
//!
//! When the `gpu` feature is enabled, CUDA kernels are used.
//! Otherwise, CPU fallback implementations are provided.

pub mod processor;
pub mod transform;

pub use processor::GpuProcessor;

/// Check if CUDA is available on this system.
pub fn is_cuda_available() -> bool {
    #[cfg(feature = "gpu")]
    {
        match cudarc::driver::CudaDevice::new(0) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    #[cfg(not(feature = "gpu"))]
    {
        false
    }
}
