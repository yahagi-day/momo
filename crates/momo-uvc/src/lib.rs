//! UVC (USB Video Class) input support.
//!
//! Linux: v4l2
//! Windows: MediaFoundation

/// Enumerate available UVC devices (stub).
pub fn enumerate_devices() -> Vec<String> {
    tracing::warn!("UVC enumeration not yet implemented");
    Vec::new()
}
