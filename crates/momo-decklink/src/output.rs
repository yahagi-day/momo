use momo_core::error::{Error, Result};
use momo_core::frame::Frame;
use momo_core::types::{DisplayMode, PixelFormat};

use crate::conversions::{display_mode_to_bmd, pixel_format_to_bmd};
use crate::ffi::decklink_ffi;
use crate::VideoOutput;

pub struct DeckLinkOutput {
    device_index: u32,
    display_mode: DisplayMode,
    pixel_format: PixelFormat,
    system: Option<cxx::UniquePtr<decklink_ffi::DeckLinkSystem>>,
    player: Option<cxx::UniquePtr<decklink_ffi::DeckLinkOutputPlayer>>,
}

impl DeckLinkOutput {
    pub fn new(device_index: u32, display_mode: DisplayMode, pixel_format: PixelFormat) -> Self {
        Self {
            device_index,
            display_mode,
            pixel_format,
            system: None,
            player: None,
        }
    }
}

impl VideoOutput for DeckLinkOutput {
    fn start(&mut self) -> Result<()> {
        let system = decklink_ffi::create_system();
        if system.is_null() || !system.is_api_present() {
            return Err(Error::DeckLink("DeckLink API not available".into()));
        }

        let bmd_mode = display_mode_to_bmd(self.display_mode);
        let bmd_fmt = pixel_format_to_bmd(self.pixel_format);

        let mut player =
            decklink_ffi::create_player(&system, self.device_index, bmd_mode, bmd_fmt);
        if player.is_null() {
            return Err(Error::DeviceNotFound(format!(
                "DeckLink output device {} not found",
                self.device_index
            )));
        }

        if !player.as_mut().unwrap().start() {
            return Err(Error::DeckLink("failed to start DeckLink output".into()));
        }

        self.system = Some(system);
        self.player = Some(player);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(ref mut player) = self.player {
            player.as_mut().unwrap().stop();
        }
        self.player = None;
        self.system = None;
        Ok(())
    }

    fn send_frame(&mut self, frame: &Frame) -> Result<()> {
        match self.player {
            Some(ref mut player) => {
                if !player.as_mut().unwrap().send_frame(&frame.data) {
                    return Err(Error::DeckLink("failed to send frame".into()));
                }
                Ok(())
            }
            None => Err(Error::Pipeline("output not started".into())),
        }
    }
}
