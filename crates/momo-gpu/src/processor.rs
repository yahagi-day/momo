use momo_core::error::Result;
use momo_core::frame::Frame;
use momo_core::types::{OutputTransform, PixelFormat, Resolution};

use crate::transform::{crop_uyvy, flip_uyvy, scale_uyvy_nearest};

/// GPU processor for per-output frame transformation.
///
/// When the `gpu` feature is enabled and CUDA is available,
/// uses GPU kernels for crop/scale/flip. Otherwise falls back to CPU.
pub struct GpuProcessor {
    #[cfg(feature = "gpu")]
    cuda: Option<crate::cuda::CudaProcessor>,
}

impl GpuProcessor {
    pub fn new() -> Self {
        #[cfg(feature = "gpu")]
        {
            let cuda = match std::panic::catch_unwind(crate::cuda::CudaProcessor::new) {
                Ok(Ok(ctx)) => {
                    tracing::info!("CUDA GPU processing enabled");
                    Some(ctx)
                }
                Ok(Err(e)) => {
                    tracing::warn!("CUDA not available, using CPU fallback: {e}");
                    None
                }
                Err(_) => {
                    tracing::warn!("CUDA driver not found, using CPU fallback");
                    None
                }
            };
            Self { cuda }
        }
        #[cfg(not(feature = "gpu"))]
        {
            tracing::info!("GPU processing using CPU fallback");
            Self {}
        }
    }

    /// Apply crop → scale → flip transform to a frame.
    ///
    /// Only UYVY pixel format is supported. The output frame has
    /// the specified `output_resolution` and the same pixel format.
    pub fn process(
        &self,
        input: &Frame,
        transform: &OutputTransform,
        output_resolution: Resolution,
    ) -> Result<Frame> {
        assert_eq!(input.format, PixelFormat::Uyvy, "only UYVY is supported");

        #[cfg(feature = "gpu")]
        if let Some(ref cuda) = self.cuda {
            return self.process_cuda(cuda, input, transform, output_resolution);
        }

        self.process_cpu(input, transform, output_resolution)
    }

    fn process_cpu(
        &self,
        input: &Frame,
        transform: &OutputTransform,
        output_resolution: Resolution,
    ) -> Result<Frame> {
        use std::sync::Arc;

        let mut data = std::borrow::Cow::Borrowed(&input.data[..]);
        let mut current_res = input.resolution;

        // 1. Crop
        if let Some(crop) = &transform.crop {
            data = std::borrow::Cow::Owned(crop_uyvy(&data, current_res, crop)?);
            current_res = Resolution {
                width: crop.width & !1,
                height: crop.height,
            };
        }

        // 2. Scale (only if resolution differs)
        if current_res != output_resolution {
            data = std::borrow::Cow::Owned(scale_uyvy_nearest(
                &data,
                current_res,
                output_resolution,
            ));
            current_res = output_resolution;
        }

        // 3. Flip
        if transform.flip.horizontal || transform.flip.vertical {
            data = std::borrow::Cow::Owned(flip_uyvy(
                &data,
                current_res,
                transform.flip.horizontal,
                transform.flip.vertical,
            ));
        }

        let out_data = match data {
            std::borrow::Cow::Borrowed(_) => input.data.clone(), // Arc clone, zero copy
            std::borrow::Cow::Owned(v) => Arc::new(v),
        };

        Ok(Frame {
            data: out_data,
            resolution: current_res,
            format: input.format,
            timestamp_ns: input.timestamp_ns,
            sequence: input.sequence,
        })
    }

    #[cfg(feature = "gpu")]
    fn process_cuda(
        &self,
        cuda: &crate::cuda::CudaProcessor,
        input: &Frame,
        transform: &OutputTransform,
        output_resolution: Resolution,
    ) -> Result<Frame> {
        let mut data = (*input.data).clone();
        let mut current_res = input.resolution;

        // 1. Crop
        if let Some(crop) = &transform.crop {
            let crop_x = crop.x & !1;
            let crop_w = (crop.width + (crop.x - crop_x)) & !1;
            data = cuda.crop_uyvy(&data, current_res, crop_x, crop.y, crop_w, crop.height)?;
            current_res = Resolution {
                width: crop_w,
                height: crop.height,
            };
        }

        // 2. Scale
        if current_res != output_resolution {
            data = cuda.scale_uyvy(&data, current_res, output_resolution)?;
            current_res = output_resolution;
        }

        // 3. Flip
        if transform.flip.horizontal || transform.flip.vertical {
            data = cuda.flip_uyvy(
                &data,
                current_res,
                transform.flip.horizontal,
                transform.flip.vertical,
            )?;
        }

        Ok(Frame {
            data: std::sync::Arc::new(data),
            resolution: current_res,
            format: input.format,
            timestamp_ns: input.timestamp_ns,
            sequence: input.sequence,
        })
    }
}

impl Default for GpuProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use momo_core::types::CropRegion;

    fn make_test_frame(w: u32, h: u32) -> Frame {
        Frame {
            data: std::sync::Arc::new(vec![128u8; (w * h * 2) as usize]),
            resolution: Resolution { width: w, height: h },
            format: PixelFormat::Uyvy,
            timestamp_ns: 1000,
            sequence: 1,
        }
    }

    #[test]
    fn process_identity() {
        let proc = GpuProcessor::new();
        let frame = make_test_frame(8, 4);
        let transform = OutputTransform::default();
        let out_res = Resolution { width: 8, height: 4 };

        let result = proc.process(&frame, &transform, out_res).unwrap();
        assert_eq!(result.data, frame.data);
        assert_eq!(result.resolution, out_res);
        assert_eq!(result.timestamp_ns, 1000);
        assert_eq!(result.sequence, 1);
    }

    #[test]
    fn process_with_crop_and_scale() {
        let proc = GpuProcessor::new();
        let frame = make_test_frame(16, 8);
        let transform = OutputTransform {
            crop: Some(CropRegion {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            }),
            flip: Default::default(),
        };
        let out_res = Resolution { width: 4, height: 4 };

        let result = proc.process(&frame, &transform, out_res).unwrap();
        assert_eq!(result.resolution, out_res);
        assert_eq!(result.data.len(), 4 * 4 * 2);
    }

    #[test]
    fn process_with_flip() {
        let proc = GpuProcessor::new();
        let frame = make_test_frame(4, 2);
        let transform = OutputTransform {
            crop: None,
            flip: momo_core::types::FlipOptions {
                horizontal: true,
                vertical: true,
            },
        };
        let out_res = Resolution { width: 4, height: 2 };

        let result = proc.process(&frame, &transform, out_res).unwrap();
        assert_eq!(result.resolution, out_res);
        assert_eq!(result.data.len(), frame.data.len());
    }
}
