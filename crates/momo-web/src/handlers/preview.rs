use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use tokio::sync::broadcast;

use momo_core::frame::Frame;
use momo_pipeline::preview::encode_preview;

use crate::state::AppState;

async fn mjpeg_stream_from_frames(
    mut rx: broadcast::Receiver<Arc<Frame>>,
    state: &AppState,
) -> Response {
    let preview_config = {
        let pipeline = state.pipeline.read().await;
        pipeline
            .config()
            .map(|c| c.preview.clone())
            .unwrap_or_default()
    };

    let stream = async_stream::stream! {
        loop {
            let frame = match rx.recv().await {
                Ok(frame) => frame,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            };
            let pc = preview_config.clone();
            let jpeg = tokio::task::spawn_blocking(move || encode_preview(&frame, &pc)).await;
            if let Ok(Ok(jpeg)) = jpeg {
                let header = format!(
                    "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                    jpeg.len()
                );
                yield Ok::<_, std::io::Error>(Bytes::from(header));
                yield Ok(Bytes::from(jpeg));
                yield Ok(Bytes::from("\r\n"));
            }
        }
    };

    Response::builder()
        .header(
            "Content-Type",
            "multipart/x-mixed-replace; boundary=frame",
        )
        .body(Body::from_stream(stream))
        .unwrap()
}

pub async fn preview_input(State(state): State<AppState>) -> Response {
    let rx = {
        let pipeline = state.pipeline.read().await;
        pipeline.subscribe_preview()
    };

    mjpeg_stream_from_frames(rx, &state).await
}

pub async fn preview_output(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let rx = {
        let pipeline = state.pipeline.read().await;
        pipeline.subscribe_output_preview(&id)
    };

    let Some(rx) = rx else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(format!("output '{id}' not found or pipeline not running")))
            .unwrap();
    };

    mjpeg_stream_from_frames(rx, &state).await
}
