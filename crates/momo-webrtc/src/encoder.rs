//! H.264 encoder wrapper using OpenH264.

use momo_core::error::{Error, Result};

use openh264::encoder::{Encoder, EncoderConfig, RateControlMode};
use openh264::formats::YUVSlices;
use openh264::OpenH264API;

/// An encoded video packet ready for WebRTC transmission.
pub struct EncodedPacket {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
}

/// H.264 encoder wrapping OpenH264.
pub struct H264Encoder {
    encoder: Encoder,
    width: u32,
    height: u32,
}

impl H264Encoder {
    /// Create a new H.264 encoder.
    pub fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        let config = EncoderConfig::new()
            .set_bitrate_bps(2_000_000)
            .max_frame_rate(fps as f32)
            .rate_control_mode(RateControlMode::Bitrate);

        let api = OpenH264API::from_source();
        let encoder = Encoder::with_api_config(api, config).map_err(|e| {
            Error::Pipeline(format!("failed to create H.264 encoder: {e}"))
        })?;

        tracing::info!(width, height, fps, "H.264 encoder created (OpenH264)");

        Ok(Self {
            encoder,
            width,
            height,
        })
    }

    /// Encode an NV12 frame, returning the encoded packet.
    ///
    /// OpenH264 expects I420 (YUV420P), so we convert NV12 → I420 first.
    pub fn encode(&mut self, nv12_data: &[u8]) -> Result<Option<EncodedPacket>> {
        let w = self.width as usize;
        let h = self.height as usize;
        let y_size = w * h;
        let uv_size = y_size / 4;

        // NV12 → I420: deinterleave UV plane
        let y_plane = &nv12_data[..y_size];
        let mut u_plane = vec![0u8; uv_size];
        let mut v_plane = vec![0u8; uv_size];
        let uv_interleaved = &nv12_data[y_size..y_size + uv_size * 2];
        for i in 0..uv_size {
            u_plane[i] = uv_interleaved[i * 2];
            v_plane[i] = uv_interleaved[i * 2 + 1];
        }

        let half_w = w / 2;
        let yuv = YUVSlices::new(
            (y_plane, &u_plane, &v_plane),
            (w, h),
            (w, half_w, half_w),
        );

        let bitstream = self.encoder.encode(&yuv).map_err(|e| {
            Error::Pipeline(format!("H.264 encode error: {e}"))
        })?;

        let data = bitstream.to_vec();
        if data.is_empty() {
            return Ok(None);
        }

        let is_keyframe = bitstream.frame_type() == openh264::encoder::FrameType::IDR
            || bitstream.frame_type() == openh264::encoder::FrameType::I;

        Ok(Some(EncodedPacket { data, is_keyframe }))
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
