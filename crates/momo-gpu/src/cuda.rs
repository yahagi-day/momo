//! CUDA-accelerated UYVY frame transforms via cudarc.
//!
//! PTX is compiled from .cu kernels at build time by nvcc (see build.rs),
//! then loaded at runtime via cudarc.

use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig, PushKernelArg};
use cudarc::nvrtc::Ptx;

use momo_core::error::{Error, Result};
use momo_core::types::Resolution;

const CROP_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/crop.ptx"));
const SCALE_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/scale.ptx"));
const FLIP_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/flip.ptx"));

const BLOCK_SIZE: u32 = 16;

pub struct CudaProcessor {
    stream: Arc<CudaStream>,
    crop_func: CudaFunction,
    scale_func: CudaFunction,
    flip_func: CudaFunction,
}

impl CudaProcessor {
    pub fn new() -> Result<Self> {
        let ctx = CudaContext::new(0)
            .map_err(|e| Error::Gpu(format!("CUDA context init failed: {e}")))?;
        let stream = ctx.default_stream();

        let crop_module = ctx
            .load_module(Ptx::from_src(CROP_PTX))
            .map_err(|e| Error::Gpu(format!("crop module load failed: {e}")))?;
        let crop_func = crop_module
            .load_function("crop_uyvy")
            .map_err(|e| Error::Gpu(format!("crop function load failed: {e}")))?;

        let scale_module = ctx
            .load_module(Ptx::from_src(SCALE_PTX))
            .map_err(|e| Error::Gpu(format!("scale module load failed: {e}")))?;
        let scale_func = scale_module
            .load_function("scale_uyvy")
            .map_err(|e| Error::Gpu(format!("scale function load failed: {e}")))?;

        let flip_module = ctx
            .load_module(Ptx::from_src(FLIP_PTX))
            .map_err(|e| Error::Gpu(format!("flip module load failed: {e}")))?;
        let flip_func = flip_module
            .load_function("flip_uyvy")
            .map_err(|e| Error::Gpu(format!("flip function load failed: {e}")))?;

        Ok(Self {
            stream,
            crop_func,
            scale_func,
            flip_func,
        })
    }

    /// Crop a UYVY frame on the GPU.
    /// `crop_x` and `crop_w` must be 2-pixel aligned.
    pub fn crop_uyvy(
        &self,
        data: &[u8],
        src_res: Resolution,
        crop_x: u32,
        crop_y: u32,
        crop_w: u32,
        crop_h: u32,
    ) -> Result<Vec<u8>> {
        let src_dev = self
            .stream
            .memcpy_stod(data)
            .map_err(|e| Error::Gpu(format!("htod copy failed: {e}")))?;
        let dst_size = (crop_w * crop_h * 2) as usize;
        let mut dst_dev = self
            .stream
            .alloc_zeros::<u8>(dst_size)
            .map_err(|e| Error::Gpu(format!("alloc failed: {e}")))?;

        let config = LaunchConfig {
            grid_dim: (
                crop_w.div_ceil(BLOCK_SIZE),
                crop_h.div_ceil(BLOCK_SIZE),
                1,
            ),
            block_dim: (BLOCK_SIZE, BLOCK_SIZE, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            self.stream
                .launch_builder(&self.crop_func)
                .arg(&src_dev)
                .arg(&mut dst_dev)
                .arg(&(src_res.width as i32))
                .arg(&(src_res.height as i32))
                .arg(&(crop_x as i32))
                .arg(&(crop_y as i32))
                .arg(&(crop_w as i32))
                .arg(&(crop_h as i32))
                .launch(config)
                .map_err(|e| Error::Gpu(format!("crop kernel launch failed: {e}")))?;
        }

        self.stream
            .memcpy_dtov(&dst_dev)
            .map_err(|e| Error::Gpu(format!("dtoh copy failed: {e}")))
    }

    /// Scale a UYVY frame on the GPU using nearest-neighbor.
    pub fn scale_uyvy(
        &self,
        data: &[u8],
        src_res: Resolution,
        dst_res: Resolution,
    ) -> Result<Vec<u8>> {
        let src_dev = self
            .stream
            .memcpy_stod(data)
            .map_err(|e| Error::Gpu(format!("htod copy failed: {e}")))?;
        let dst_size = (dst_res.width * dst_res.height * 2) as usize;
        let mut dst_dev = self
            .stream
            .alloc_zeros::<u8>(dst_size)
            .map_err(|e| Error::Gpu(format!("alloc failed: {e}")))?;

        // Grid covers macro-pixels (width/2) × height
        let dst_macro_w = dst_res.width / 2;
        let config = LaunchConfig {
            grid_dim: (
                dst_macro_w.div_ceil(BLOCK_SIZE),
                dst_res.height.div_ceil(BLOCK_SIZE),
                1,
            ),
            block_dim: (BLOCK_SIZE, BLOCK_SIZE, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            self.stream
                .launch_builder(&self.scale_func)
                .arg(&src_dev)
                .arg(&mut dst_dev)
                .arg(&(src_res.width as i32))
                .arg(&(src_res.height as i32))
                .arg(&(dst_res.width as i32))
                .arg(&(dst_res.height as i32))
                .launch(config)
                .map_err(|e| Error::Gpu(format!("scale kernel launch failed: {e}")))?;
        }

        self.stream
            .memcpy_dtov(&dst_dev)
            .map_err(|e| Error::Gpu(format!("dtoh copy failed: {e}")))
    }

    /// Flip a UYVY frame on the GPU.
    pub fn flip_uyvy(
        &self,
        data: &[u8],
        resolution: Resolution,
        horizontal: bool,
        vertical: bool,
    ) -> Result<Vec<u8>> {
        let src_dev = self
            .stream
            .memcpy_stod(data)
            .map_err(|e| Error::Gpu(format!("htod copy failed: {e}")))?;
        let dst_size = data.len();
        let mut dst_dev = self
            .stream
            .alloc_zeros::<u8>(dst_size)
            .map_err(|e| Error::Gpu(format!("alloc failed: {e}")))?;

        // Grid covers macro-pixels (width/2) × height
        let macro_w = resolution.width / 2;
        let config = LaunchConfig {
            grid_dim: (
                macro_w.div_ceil(BLOCK_SIZE),
                resolution.height.div_ceil(BLOCK_SIZE),
                1,
            ),
            block_dim: (BLOCK_SIZE, BLOCK_SIZE, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            self.stream
                .launch_builder(&self.flip_func)
                .arg(&src_dev)
                .arg(&mut dst_dev)
                .arg(&(resolution.width as i32))
                .arg(&(resolution.height as i32))
                .arg(&(horizontal as i32))
                .arg(&(vertical as i32))
                .launch(config)
                .map_err(|e| Error::Gpu(format!("flip kernel launch failed: {e}")))?;
        }

        self.stream
            .memcpy_dtov(&dst_dev)
            .map_err(|e| Error::Gpu(format!("dtoh copy failed: {e}")))
    }
}
