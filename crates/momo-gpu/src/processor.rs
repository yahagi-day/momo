use momo_core::error::Result;
use momo_core::frame::Frame;
use momo_core::types::{OutputTransform, PixelFormat, Resolution};

use crate::transform::{crop_uyvy, flip_uyvy, scale_uyvy_nearest};

/// GPU processor for per-output frame transformation.
///
/// Currently uses CPU fallback for all operations.
/// When the `gpu` feature is enabled, CUDA kernels will be used instead.
pub struct GpuProcessor {
    #[cfg(feature = "gpu")]
    _cuda_available: bool,
}

impl GpuProcessor {
    pub fn new() -> Self {
        #[cfg(feature = "gpu")]
        {
            let available = crate::is_cuda_available();
            if available {
                tracing::info!("CUDA GPU processing enabled");
            } else {
                tracing::warn!("CUDA not available, using CPU fallback");
            }
            Self {
                _cuda_available: available,
            }
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

        Ok(Frame {
            data: data.into_owned(),
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
            data: vec![128u8; (w * h * 2) as usize],
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
