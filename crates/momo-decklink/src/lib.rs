//! DeckLink FFI bindings via cxx.
//!
//! This crate will provide:
//! - Device enumeration
//! - Video input capture
//! - Video output scheduling
//! - Hot-unplug notification

use momo_core::frame::Frame;
use momo_core::types::DeviceStatus;

/// Information about a DeckLink device.
#[derive(Debug, Clone)]
pub struct DeckLinkDevice {
    pub index: u32,
    pub name: String,
    pub status: DeviceStatus,
}

/// Trait for video input sources.
pub trait VideoInput: Send {
    /// Start capturing frames.
    fn start(&mut self) -> momo_core::error::Result<()>;
    /// Stop capturing.
    fn stop(&mut self) -> momo_core::error::Result<()>;
    /// Whether the source is currently capturing.
    fn is_capturing(&self) -> bool;
}

/// Trait for video output destinations.
pub trait VideoOutput: Send {
    /// Start output.
    fn start(&mut self) -> momo_core::error::Result<()>;
    /// Stop output.
    fn stop(&mut self) -> momo_core::error::Result<()>;
    /// Schedule a frame for output.
    fn send_frame(&mut self, frame: &Frame) -> momo_core::error::Result<()>;
}

/// Enumerate available DeckLink devices (stub).
pub fn enumerate_devices() -> Vec<DeckLinkDevice> {
    tracing::warn!("DeckLink enumeration not yet implemented");
    Vec::new()
}
