//! Frame routing pipeline: input → GPU → outputs.
//!
//! Manages the data flow from a single video input through GPU
//! processing to multiple DeckLink outputs and preview streams.

pub mod event;
pub mod input;
pub mod mock_input;
pub mod pipeline;
pub mod preview;

pub use event::PipelineEvent;
pub use pipeline::Pipeline;
