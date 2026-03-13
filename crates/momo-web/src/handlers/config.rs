use axum::extract::{Path, State};
use axum::Json;
use momo_core::config::Config;
use momo_core::types::OutputTransform;
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

pub async fn get_config(
    State(state): State<AppState>,
) -> Result<Json<Config>, AppError> {
    let pipeline = state.pipeline.read().await;
    let config = pipeline
        .config()
        .cloned()
        .ok_or_else(|| momo_core::Error::Config("no configuration set".into()))?;
    Ok(Json(config))
}

pub async fn put_config(
    State(state): State<AppState>,
    Json(config): Json<Config>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut pipeline = state.pipeline.write().await;
    pipeline.set_config(config)?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn patch_output(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(transform): Json<OutputTransform>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut pipeline = state.pipeline.write().await;
    pipeline.update_output(&id, transform)?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(Deserialize)]
pub struct LoadRequest {
    pub path: String,
}

pub async fn save_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pipeline = state.pipeline.read().await;
    pipeline.save_config()?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn load_config(
    State(state): State<AppState>,
    Json(req): Json<LoadRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut pipeline = state.pipeline.write().await;
    pipeline.load_config(std::path::Path::new(&req.path))?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
