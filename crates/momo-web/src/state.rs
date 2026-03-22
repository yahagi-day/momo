use std::sync::Arc;

use momo_pipeline::Pipeline;
use tokio::sync::RwLock;

#[cfg(feature = "webrtc")]
use momo_webrtc::WebRtcManager;

#[derive(Clone)]
pub struct AppState {
    pub pipeline: Arc<RwLock<Pipeline>>,
    #[cfg(feature = "webrtc")]
    pub webrtc_manager: Arc<WebRtcManager>,
}

impl AppState {
    pub fn new(pipeline: Pipeline) -> Self {
        #[cfg(feature = "webrtc")]
        {
            let preview_config = pipeline
                .config()
                .map(|c| c.preview.clone())
                .unwrap_or_default();
            let pipeline_arc = Arc::new(RwLock::new(pipeline));
            let pipeline_ref = pipeline_arc.clone();
            let subscribe_fn: momo_webrtc::manager::SubscribeFn =
                Arc::new(move |stream_id: &str| {
                    let pipeline = pipeline_ref.blocking_read();
                    if stream_id == "input" {
                        Some(pipeline.subscribe_raw_preview())
                    } else {
                        pipeline.subscribe_raw_output_preview(stream_id)
                    }
                });
            let manager = Arc::new(WebRtcManager::new(subscribe_fn, preview_config));
            Self {
                pipeline: pipeline_arc,
                webrtc_manager: manager,
            }
        }

        #[cfg(not(feature = "webrtc"))]
        Self {
            pipeline: Arc::new(RwLock::new(pipeline)),
        }
    }
}
