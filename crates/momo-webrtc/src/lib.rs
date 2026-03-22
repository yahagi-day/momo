//! WebRTC preview streaming with H.264 support.
//!
//! Feature-gated: requires `--features webrtc` to enable str0m + OpenH264.
//! Without the feature, provides signal types only.

#[cfg(feature = "webrtc")]
pub mod encoder;
#[cfg(feature = "webrtc")]
pub mod session;
#[cfg(feature = "webrtc")]
pub mod manager;

pub mod convert;
pub mod signal;

#[cfg(feature = "webrtc")]
pub use manager::{SessionHandle, WebRtcManager};

/// Check if WebRTC support is available in this build.
pub fn is_webrtc_available() -> bool {
    cfg!(feature = "webrtc")
}
