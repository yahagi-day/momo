//! Web server: axum REST API + WebSocket + MJPEG preview.

pub mod embedded_ui;
pub mod error;
pub mod handlers;
pub mod state;

use axum::routing::{get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;

use state::AppState;

/// Build the axum router with all API routes.
///
/// The UI HTML is embedded at compile time via build.rs:
/// - If `frontend/dist/index.html` exists during build, the SolidJS SPA is embedded.
/// - Otherwise, a self-contained fallback HTML is embedded.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/config", get(handlers::config::get_config).put(handlers::config::put_config))
        .route("/api/config/output/{id}", put(handlers::config::patch_output))
        .route("/api/config/save", post(handlers::config::save_config))
        .route("/api/config/load", post(handlers::config::load_config))
        .route("/api/devices", get(handlers::devices::get_devices))
        .route("/api/status", get(handlers::pipeline::get_status))
        .route("/api/pipeline/start", post(handlers::pipeline::start_pipeline))
        .route("/api/pipeline/stop", post(handlers::pipeline::stop_pipeline))
        .route("/ws/status", get(handlers::ws::ws_handler))
        .route("/api/preview/input", get(handlers::preview::preview_input))
        .route("/api/preview/output/{id}", get(handlers::preview::preview_output))
        .fallback(embedded_ui::index_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use momo_core::config::{
        Config, InputSource, OutputConfig, PreviewConfig, WebConfig,
    };
    use momo_core::types::{DisplayMode, OutputTransform, PixelFormat};
    use momo_pipeline::Pipeline;
    use tower::ServiceExt;

    fn mock_config() -> Config {
        Config {
            input: InputSource::Mock {
                width: 320,
                height: 240,
                fps: 10,
            },
            outputs: vec![OutputConfig {
                id: "out1".into(),
                name: "Output 1".into(),
                device_index: 0,
                display_mode: DisplayMode::Hd1080p5994,
                pixel_format: PixelFormat::Uyvy,
                transform: OutputTransform::default(),
                enabled: true,
            }],
            preview: PreviewConfig::default(),
            web: WebConfig::default(),
        }
    }

    fn test_state() -> AppState {
        let mut pipeline = Pipeline::new();
        pipeline.set_config(mock_config()).unwrap();
        AppState::new(pipeline)
    }

    #[tokio::test]
    async fn get_status_returns_stopped() {
        let app = build_router(test_state());
        let req = Request::get("/api/status").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["state"], "Stopped");
    }

    #[tokio::test]
    async fn get_config_returns_config() {
        let app = build_router(test_state());
        let req = Request::get("/api/config").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["outputs"][0]["id"], "out1");
    }

    #[tokio::test]
    async fn get_config_no_config_returns_400() {
        let state = AppState::new(Pipeline::new());
        let app = build_router(state);
        let req = Request::get("/api/config").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn put_config_sets_config() {
        let state = test_state();
        let app = build_router(state);
        let config_json = serde_json::to_string(&mock_config()).unwrap();
        let req = Request::put("/api/config")
            .header("content-type", "application/json")
            .body(Body::from(config_json))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_devices_returns_array() {
        let app = build_router(test_state());
        let req = Request::get("/api/devices").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
    }

    #[tokio::test]
    async fn start_stop_pipeline() {
        let state = test_state();

        // Start
        let app = build_router(state.clone());
        let req = Request::post("/api/pipeline/start")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Stop
        let app = build_router(state);
        let req = Request::post("/api/pipeline/stop")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn stop_when_stopped_returns_conflict() {
        let app = build_router(test_state());
        let req = Request::post("/api/pipeline/stop")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn preview_output_returns_404_when_stopped() {
        let app = build_router(test_state());
        let req = Request::get("/api/preview/output/out1")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn preview_output_returns_mjpeg_when_running() {
        let state = test_state();
        {
            let mut pipeline = state.pipeline.write().await;
            pipeline.start().unwrap();
        }

        let app = build_router(state.clone());
        let req = Request::get("/api/preview/output/out1")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(content_type.contains("multipart/x-mixed-replace"));

        {
            let mut pipeline = state.pipeline.write().await;
            pipeline.stop().unwrap();
        }
    }

    #[tokio::test]
    async fn preview_input_returns_mjpeg_content_type() {
        let state = test_state();

        // Start pipeline first so preview frames are generated
        {
            let mut pipeline = state.pipeline.write().await;
            pipeline.start().unwrap();
        }

        let app = build_router(state.clone());
        let req = Request::get("/api/preview/input")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(content_type.contains("multipart/x-mixed-replace"));

        // Stop pipeline
        {
            let mut pipeline = state.pipeline.write().await;
            pipeline.stop().unwrap();
        }
    }

    #[tokio::test]
    async fn update_output_transform() {
        let state = test_state();
        let app = build_router(state);
        let transform = serde_json::json!({
            "flip": { "horizontal": true, "vertical": false }
        });
        let req = Request::put("/api/config/output/out1")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&transform).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
