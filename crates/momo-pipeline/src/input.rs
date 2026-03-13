use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crossbeam_channel::Sender;
use momo_core::config::InputSource;
use momo_core::error::{Error, Result};
use momo_core::frame::Frame;

use crate::mock_input::MockInput;

pub enum InputDriver {
    Mock(MockInput),
}

impl InputDriver {
    pub fn from_config(input: &InputSource) -> Result<Self> {
        match input {
            InputSource::Mock { width, height, fps } => {
                Ok(InputDriver::Mock(MockInput::new(*width, *height, *fps)))
            }
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
        }
    }
}
