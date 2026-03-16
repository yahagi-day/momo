use momo_core::types::{DisplayMode, PixelFormat};

/// Convert a `DisplayMode` to the corresponding BMDDisplayMode constant.
pub fn display_mode_to_bmd(mode: DisplayMode) -> u32 {
    match mode {
        DisplayMode::Hd720p50 => 0x68703530,   // 'hp50'
        DisplayMode::Hd720p5994 => 0x68703539,  // 'hp59'
        DisplayMode::Hd720p60 => 0x68703630,    // 'hp60'
        DisplayMode::Hd1080i50 => 0x48693530,   // 'Hi50'
        DisplayMode::Hd1080i5994 => 0x48693539,  // 'Hi59'
        DisplayMode::Hd1080p24 => 0x32347073,   // '24ps'
        DisplayMode::Hd1080p25 => 0x48703235,   // 'Hp25'
        DisplayMode::Hd1080p2997 => 0x48703239,  // 'Hp29'
        DisplayMode::Hd1080p30 => 0x48703330,   // 'Hp30'
        DisplayMode::Hd1080p50 => 0x48703530,   // 'Hp50'
        DisplayMode::Hd1080p5994 => 0x48703539,  // 'Hp59'
        DisplayMode::Hd1080p60 => 0x48703630,   // 'Hp60'
        DisplayMode::Uhd2160p24 => 0x346B3234,  // '4k24'
        DisplayMode::Uhd2160p25 => 0x346B3235,  // '4k25'
        DisplayMode::Uhd2160p2997 => 0x346B3239, // '4k29'
        DisplayMode::Uhd2160p30 => 0x346B3330,  // '4k30'
        DisplayMode::Uhd2160p50 => 0x346B3530,  // '4k50'
        DisplayMode::Uhd2160p5994 => 0x346B3539, // '4k59'
        DisplayMode::Uhd2160p60 => 0x346B3630,  // '4k60'
    }
}

/// Convert a BMDDisplayMode constant to the corresponding `DisplayMode`.
/// Returns `None` for unknown modes.
pub fn bmd_to_display_mode(bmd: u32) -> Option<DisplayMode> {
    match bmd {
        0x68703530 => Some(DisplayMode::Hd720p50),
        0x68703539 => Some(DisplayMode::Hd720p5994),
        0x68703630 => Some(DisplayMode::Hd720p60),
        0x48693530 => Some(DisplayMode::Hd1080i50),
        0x48693539 => Some(DisplayMode::Hd1080i5994),
        0x32347073 => Some(DisplayMode::Hd1080p24),
        0x48703235 => Some(DisplayMode::Hd1080p25),
        0x48703239 => Some(DisplayMode::Hd1080p2997),
        0x48703330 => Some(DisplayMode::Hd1080p30),
        0x48703530 => Some(DisplayMode::Hd1080p50),
        0x48703539 => Some(DisplayMode::Hd1080p5994),
        0x48703630 => Some(DisplayMode::Hd1080p60),
        0x346B3234 => Some(DisplayMode::Uhd2160p24),
        0x346B3235 => Some(DisplayMode::Uhd2160p25),
        0x346B3239 => Some(DisplayMode::Uhd2160p2997),
        0x346B3330 => Some(DisplayMode::Uhd2160p30),
        0x346B3530 => Some(DisplayMode::Uhd2160p50),
        0x346B3539 => Some(DisplayMode::Uhd2160p5994),
        0x346B3630 => Some(DisplayMode::Uhd2160p60),
        _ => None,
    }
}

/// Convert a `PixelFormat` to the corresponding BMDPixelFormat constant.
pub fn pixel_format_to_bmd(fmt: PixelFormat) -> u32 {
    match fmt {
        PixelFormat::Uyvy => 0x32767579, // '2vuy'
        PixelFormat::Bgra => 0x42475241, // 'BGRA'
        PixelFormat::V210 => 0x76323130, // 'v210'
    }
}

/// Convert a BMDPixelFormat constant to the corresponding `PixelFormat`.
/// Returns `None` for unknown formats.
pub fn bmd_to_pixel_format(bmd: u32) -> Option<PixelFormat> {
    match bmd {
        0x32767579 => Some(PixelFormat::Uyvy),
        0x42475241 => Some(PixelFormat::Bgra),
        0x76323130 => Some(PixelFormat::V210),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_mode_roundtrip() {
        let modes = [
            DisplayMode::Hd720p50,
            DisplayMode::Hd720p5994,
            DisplayMode::Hd720p60,
            DisplayMode::Hd1080i50,
            DisplayMode::Hd1080i5994,
            DisplayMode::Hd1080p24,
            DisplayMode::Hd1080p25,
            DisplayMode::Hd1080p2997,
            DisplayMode::Hd1080p30,
            DisplayMode::Hd1080p50,
            DisplayMode::Hd1080p5994,
            DisplayMode::Hd1080p60,
            DisplayMode::Uhd2160p24,
            DisplayMode::Uhd2160p25,
            DisplayMode::Uhd2160p2997,
            DisplayMode::Uhd2160p30,
            DisplayMode::Uhd2160p50,
            DisplayMode::Uhd2160p5994,
            DisplayMode::Uhd2160p60,
        ];
        for mode in modes {
            let bmd = display_mode_to_bmd(mode);
            let back = bmd_to_display_mode(bmd).expect("roundtrip failed");
            assert_eq!(mode, back, "roundtrip failed for {:?} (0x{:08X})", mode, bmd);
        }
    }

    #[test]
    fn pixel_format_roundtrip() {
        for fmt in [PixelFormat::Uyvy, PixelFormat::Bgra, PixelFormat::V210] {
            let bmd = pixel_format_to_bmd(fmt);
            let back = bmd_to_pixel_format(bmd).expect("roundtrip failed");
            assert_eq!(fmt, back);
        }
    }

    #[test]
    fn unknown_bmd_display_mode() {
        assert_eq!(bmd_to_display_mode(0xDEADBEEF), None);
    }

    #[test]
    fn unknown_bmd_pixel_format() {
        assert_eq!(bmd_to_pixel_format(0xDEADBEEF), None);
    }

    #[test]
    fn known_bmd_constants() {
        assert_eq!(pixel_format_to_bmd(PixelFormat::Uyvy), 0x32767579);
        assert_eq!(pixel_format_to_bmd(PixelFormat::Bgra), 0x42475241);
        assert_eq!(pixel_format_to_bmd(PixelFormat::V210), 0x76323130);
    }
}
