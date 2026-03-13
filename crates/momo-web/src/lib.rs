//! Web server: axum REST API + WebSocket + MJPEG preview.

use axum::{routing::get, Router};

/// Build the axum router with all API routes.
pub fn build_router() -> Router {
    Router::new()
        .route("/api/status", get(status_handler))
}

async fn status_handler() -> &'static str {
    r#"{"state":"stopped"}"#
}
