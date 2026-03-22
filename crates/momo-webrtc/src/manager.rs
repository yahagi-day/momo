//! WebRTC session manager — connects sessions to the Pipeline.

use std::sync::Arc;

use momo_core::config::PreviewConfig;
use momo_core::frame::Frame;
use tokio::sync::{broadcast, mpsc};

use crate::session::run_session;
use crate::signal::{ClientMessage, ServerMessage};

/// Callback to subscribe to pipeline preview frames.
pub type SubscribeFn =
    Arc<dyn Fn(&str) -> Option<broadcast::Receiver<Arc<Frame>>> + Send + Sync>;

/// Handle for an active WebRTC session.
pub struct SessionHandle {
    pub signal_tx: mpsc::Sender<ClientMessage>,
    pub signal_rx: mpsc::Receiver<ServerMessage>,
    _task: tokio::task::JoinHandle<()>,
}

/// Manages WebRTC sessions and their lifecycle.
pub struct WebRtcManager {
    subscribe_fn: SubscribeFn,
    preview_config: PreviewConfig,
}

impl WebRtcManager {
    pub fn new(subscribe_fn: SubscribeFn, preview_config: PreviewConfig) -> Self {
        Self {
            subscribe_fn,
            preview_config,
        }
    }

    /// Create a new WebRTC session, returning a handle for signaling.
    pub fn create_session(&self) -> SessionHandle {
        let (client_tx, client_rx) = mpsc::channel::<ClientMessage>(32);
        let (server_tx, server_rx) = mpsc::channel::<ServerMessage>(32);

        let subscribe_fn = self.subscribe_fn.clone();
        let pc = self.preview_config.clone();

        let task = tokio::spawn(async move {
            run_session(
                client_rx,
                server_tx,
                subscribe_fn,
                pc.width,
                pc.height,
                pc.fps,
            )
            .await;
        });

        tracing::info!("WebRTC session created");

        SessionHandle {
            signal_tx: client_tx,
            signal_rx: server_rx,
            _task: task,
        }
    }
}
