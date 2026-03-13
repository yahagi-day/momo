use serde::{Deserialize, Serialize};

/// Pixel format for video frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 8-bit UYVY (4:2:2) — DeckLink native
    Uyvy,
    /// 8-bit BGRA (4:4:4:4)
    Bgra,
    /// 10-bit YUV (4:2:2) — DeckLink v210
    V210,
}

/// Video resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Frame rate as a rational number (e.g. 30000/1001 for 29.97fps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRate {
    pub num: u32,
    pub den: u32,
}

impl FrameRate {
    pub fn fps(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

/// A known DeckLink display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Hd1080i50,
    Hd1080i5994,
    Hd1080p24,
    Hd1080p25,
    Hd1080p2997,
    Hd1080p30,
    Hd1080p50,
    Hd1080p5994,
    Hd1080p60,
    Hd720p50,
    Hd720p5994,
    Hd720p60,
    Uhd2160p24,
    Uhd2160p25,
    Uhd2160p2997,
    Uhd2160p30,
    Uhd2160p50,
    Uhd2160p5994,
    Uhd2160p60,
}

impl DisplayMode {
    pub fn resolution(&self) -> Resolution {
        use DisplayMode::*;
        match self {
            Hd1080i50 | Hd1080i5994 | Hd1080p24 | Hd1080p25 | Hd1080p2997 | Hd1080p30
            | Hd1080p50 | Hd1080p5994 | Hd1080p60 => Resolution {
                width: 1920,
                height: 1080,
            },
            Hd720p50 | Hd720p5994 | Hd720p60 => Resolution {
                width: 1280,
                height: 720,
            },
            Uhd2160p24 | Uhd2160p25 | Uhd2160p2997 | Uhd2160p30 | Uhd2160p50
            | Uhd2160p5994 | Uhd2160p60 => Resolution {
                width: 3840,
                height: 2160,
            },
        }
    }

    pub fn frame_rate(&self) -> FrameRate {
        use DisplayMode::*;
        match self {
            Hd1080p24 | Uhd2160p24 => FrameRate { num: 24000, den: 1000 },
            Hd1080p25 | Hd1080i50 | Uhd2160p25 => FrameRate { num: 25000, den: 1000 },
            Hd1080p2997 | Hd1080i5994 | Uhd2160p2997 => FrameRate { num: 30000, den: 1001 },
            Hd1080p30 | Uhd2160p30 => FrameRate { num: 30000, den: 1000 },
            Hd720p50 | Hd1080p50 | Uhd2160p50 => FrameRate { num: 50000, den: 1000 },
            Hd720p5994 | Hd1080p5994 | Uhd2160p5994 => FrameRate { num: 60000, den: 1001 },
            Hd720p60 | Hd1080p60 | Uhd2160p60 => FrameRate { num: 60000, den: 1000 },
        }
    }
}

/// Horizontal/vertical flip options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipOptions {
    pub horizontal: bool,
    pub vertical: bool,
}

/// Crop region in pixel coordinates (relative to input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CropRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Per-output transform configuration: crop → scale → flip.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputTransform {
    /// Crop region from the input frame. `None` means full frame.
    pub crop: Option<CropRegion>,
    /// Flip after scaling.
    #[serde(default)]
    pub flip: FlipOptions,
}

/// Device status for DeckLink devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Available,
    InUse,
    Disconnected,
}

/// Pipeline state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}
