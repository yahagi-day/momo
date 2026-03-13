use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct StatusResponse {
    pub state: momo_core::PipelineState,
}

pub async fn get_status(State(state): State<AppState>) -> Json<StatusResponse> {
    let pipeline = state.pipeline.read().await;
    Json(StatusResponse {
        state: pipeline.state(),
    })
}

pub async fn start_pipeline(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut pipeline = state.pipeline.write().await;
    pipeline.start()?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn stop_pipeline(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut pipeline = state.pipeline.write().await;
    pipeline.stop()?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
