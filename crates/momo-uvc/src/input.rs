//! UVC camera input capture.

#[cfg(feature = "uvc")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "uvc")]
use std::sync::Arc;

#[cfg(feature = "uvc")]
use crossbeam_channel::Sender;
#[cfg(feature = "uvc")]
use momo_core::frame::Frame;
#[cfg(feature = "uvc")]
use momo_core::types::{PixelFormat, Resolution};

#[cfg(feature = "uvc")]
use crate::convert::yuyv_to_uyvy;

/// UVC camera input device.
#[cfg(feature = "uvc")]
pub struct UvcInput {
    device_path: String,
    width: u32,
    height: u32,
    fps: u32,
}

#[cfg(feature = "uvc")]
impl UvcInput {
    pub fn new(device_path: &str, width: u32, height: u32, fps: u32) -> Self {
        Self {
            device_path: device_path.to_string(),
            width,
            height,
            fps,
        }
    }

    pub fn start(
        self,
        tx: Sender<Frame>,
        stop_flag: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("uvc-input".into())
            .spawn(move || {
                tracing::info!(
                    "UVC input starting: {} ({}x{} @ {}fps)",
                    self.device_path,
                    self.width,
                    self.height,
                    self.fps
                );

                let requested = nokhwa::utils::RequestedFormat::new::<
                    nokhwa::pixel_format::RgbFormat,
                >(nokhwa::utils::RequestedFormatType::Closest(
                    nokhwa::utils::CameraFormat::new(
                        nokhwa::utils::Resolution::new(self.width, self.height),
                        nokhwa::utils::FrameFormat::YUYV,
                        self.fps,
                    ),
                ));

                let index = nokhwa::utils::CameraIndex::String(self.device_path.clone());
                let mut camera = match nokhwa::Camera::new(index, requested) {
                    Ok(cam) => cam,
                    Err(e) => {
                        tracing::error!("failed to open UVC camera: {e}");
                        return;
                    }
                };

                if let Err(e) = camera.open_stream() {
                    tracing::error!("failed to start UVC stream: {e}");
                    return;
                }

                let mut sequence = 0u64;
                let start = std::time::Instant::now();

                while !stop_flag.load(Ordering::Relaxed) {
                    match camera.frame_raw() {
                        Ok(buf) => {
                            let data = yuyv_to_uyvy(buf.buffer());
                            let frame = Frame {
                                data,
                                resolution: Resolution {
                                    width: self.width,
                                    height: self.height,
                                },
                                format: PixelFormat::Uyvy,
                                timestamp_ns: start.elapsed().as_nanos() as u64,
                                sequence,
                            };
                            sequence += 1;
                            if tx.send(frame).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("UVC frame error: {e}");
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }

                let _ = camera.stop_stream();
                tracing::info!("UVC input stopped");
            })
            .expect("failed to spawn UVC input thread")
    }
}
