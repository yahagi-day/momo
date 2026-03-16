//! UVC (USB Video Class) input support.
//!
//! When the `uvc` feature is enabled, provides camera enumeration
//! and capture via the `nokhwa` crate.
//!
//! Linux: v4l2, Windows: MediaFoundation

pub mod convert;
#[cfg(feature = "uvc")]
pub mod input;

/// Enumerate available UVC devices.
///
/// Returns device paths/identifiers. Without the `uvc` feature, returns empty.
pub fn enumerate_devices() -> Vec<String> {
    #[cfg(feature = "uvc")]
    {
        match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
            Ok(devices) => devices
                .iter()
                .map(|d| format!("{}: {}", d.index(), d.human_name()))
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
