//! UVC (USB Video Class) input support.
//!
//! When the `uvc` feature is enabled, provides camera enumeration
//! and capture via the `nokhwa` crate.
//!
//! Linux: v4l2, Windows: MediaFoundation

pub mod convert;
#[cfg(feature = "uvc")]
pub mod input;

/// Information about an available UVC device.
pub struct UvcDevice {
    /// Numeric index of the device.
    pub index: u32,
    /// Human-readable device name.
    pub name: String,
    /// Device path/identifier for use in `InputSource::Uvc { device_path }`.
    pub path: String,
}

/// Enumerate available UVC devices.
///
/// Returns structured device info. Without the `uvc` feature, returns empty.
pub fn enumerate_devices() -> Vec<UvcDevice> {
    #[cfg(feature = "uvc")]
    {
        match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
            Ok(devices) => devices
                .iter()
                .map(|d| UvcDevice {
                    index: match d.index() {
                        nokhwa::utils::CameraIndex::Index(i) => *i,
                        _ => 0,
                    },
                    name: d.human_name().to_string(),
                    path: format!("{}", d.index()),
                })
                .collect(),
            Err(e) => {
                tracing::warn!("UVC enumeration failed: {e}");
                Vec::new()
            }
        }
    }
    #[cfg(not(feature = "uvc"))]
    {
        tracing::debug!("UVC support not enabled (compile with --features uvc)");
        Vec::new()
    }
}
