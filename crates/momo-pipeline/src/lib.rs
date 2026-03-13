//! Frame routing pipeline: input → GPU → outputs.
//!
//! Manages the data flow from a single video input through GPU
//! processing to multiple DeckLink outputs and preview streams.

use momo_core::types::PipelineState;

/// Pipeline manager that coordinates input, GPU, and outputs.
pub struct Pipeline {
    state: PipelineState,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            state: PipelineState::Stopped,
        }
    }

    pub fn state(&self) -> PipelineState {
        self.state
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
