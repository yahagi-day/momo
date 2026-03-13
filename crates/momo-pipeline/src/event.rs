use momo_core::PipelineState;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum PipelineEvent {
    StateChanged { state: PipelineState },
    FpsUpdate { fps: f64 },
    DeviceEvent { device: String, status: String },
    ConfigChanged,
    Error { message: String },
}
