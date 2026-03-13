use std::sync::Arc;

use momo_pipeline::Pipeline;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub pipeline: Arc<RwLock<Pipeline>>,
}

impl AppState {
    pub fn new(pipeline: Pipeline) -> Self {
        Self {
            pipeline: Arc::new(RwLock::new(pipeline)),
        }
    }
}
