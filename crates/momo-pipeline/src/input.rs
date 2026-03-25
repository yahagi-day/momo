use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crossbeam_channel::Sender;
use momo_core::config::InputSource;
use momo_core::error::{Error, Result};
use momo_core::frame::Frame;

use crate::mock_input::MockInput;

pub enum InputDriver {
    Mock(MockInput),
    #[cfg(feature = "decklink")]
    DeckLink(momo_decklink::input::DeckLinkInput),
    #[cfg(feature = "uvc")]
    Uvc(momo_uvc::input::UvcInput),
}

impl InputDriver {
    pub fn from_config(input: &InputSource) -> Result<Self> {
        match input {
            InputSource::Mock { width, height, fps } => {
                Ok(InputDriver::Mock(MockInput::new(*width, *height, *fps)))
            }
            #[cfg(feature = "decklink")]
            InputSource::DeckLink {
                device_index,
                display_mode,
                pixel_format,
            } => Ok(InputDriver::DeckLink(
                momo_decklink::input::DeckLinkInput::new(
                    *device_index,
                    *display_mode,
                    *pixel_format,
                ),
            )),
            #[cfg(feature = "uvc")]
            InputSource::Uvc { device_path } => {
                // Default to 1920x1080 @ 30fps for UVC cameras
                Ok(InputDriver::Uvc(momo_uvc::input::UvcInput::new(
                    device_path,
                    1920,
                    1080,
                    30,
                )))
            }
            #[allow(unreachable_patterns)]
            _ => Err(Error::Pipeline(
                "unsupported input source (hardware not available)".into(),
            )),
        }
    }

    pub fn start(
        self,
        tx: Sender<Frame>,
        stop_flag: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        match self {
            InputDriver::Mock(mock) => mock.start(tx, stop_flag),
            #[cfg(feature = "decklink")]
            InputDriver::DeckLink(dl) => dl.start(tx, stop_flag),
            #[cfg(feature = "uvc")]
            InputDriver::Uvc(uvc) => uvc.start(tx, stop_flag),
        }
    }
}
