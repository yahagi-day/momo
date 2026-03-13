use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

pub async fn preview_input(State(state): State<AppState>) -> Response {
    let rx = {
        let pipeline = state.pipeline.read().await;
        pipeline.subscribe_preview()
    };

    let stream = async_stream::stream! {
        let mut rx = rx;
        while let Ok(jpeg) = rx.recv().await {
            let header = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                jpeg.len()
            );
            yield Ok::<_, std::io::Error>(Bytes::from(header));
            yield Ok(Bytes::from(jpeg));
            yield Ok(Bytes::from("\r\n"));
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

pub async fn preview_output(Path(_id): Path<String>) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "output preview not yet implemented",
    )
}
