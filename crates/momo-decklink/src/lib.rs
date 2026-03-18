//! DeckLink FFI bindings via cxx.
//!
//! This crate provides:
//! - Device enumeration
//! - Video input capture
//! - Video output scheduling
//! - Hot-unplug notification
//!
//! The `decklink` feature enables the actual C++ FFI bindings.
//! Without it, all hardware functions return stubs.

use momo_core::frame::Frame;
use momo_core::types::DeviceStatus;

pub mod conversions;

#[cfg(feature = "decklink")]
mod ffi;
#[cfg(feature = "decklink")]
pub mod input;
#[cfg(feature = "decklink")]
pub mod output;

/// Information about a DeckLink device.
#[derive(Debug, Clone)]
pub struct DeckLinkDevice {
    pub index: u32,
    pub name: String,
    pub model_name: String,
    pub has_input: bool,
    pub has_output: bool,
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

/// Enumerate available DeckLink devices.
///
/// With the `decklink` feature enabled, this queries real hardware via the DeckLink API.
/// Without it, returns an empty list.
pub fn enumerate_devices() -> Vec<DeckLinkDevice> {
    #[cfg(feature = "decklink")]
    {
        enumerate_devices_ffi()
    }
    #[cfg(not(feature = "decklink"))]
    {
        tracing::debug!("DeckLink enumeration: feature not enabled, returning empty");
        Vec::new()
    }
}

#[cfg(feature = "decklink")]
fn enumerate_devices_ffi() -> Vec<DeckLinkDevice> {
    let system = ffi::decklink_ffi::create_system();
    if system.is_null() || !system.is_api_present() {
        tracing::warn!("DeckLink API not available (driver not installed?)");
        return Vec::new();
    }

    let bridge_devices = system.enumerate();
    bridge_devices
        .into_iter()
        .map(|d| DeckLinkDevice {
            index: d.index,
            name: d.name.clone(),
            model_name: d.model_name.clone(),
            has_input: d.has_input,
            has_output: d.has_output,
            status: DeviceStatus::Available,
        })
        .collect()
}
