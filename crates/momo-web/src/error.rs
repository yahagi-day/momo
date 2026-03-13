use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub struct AppError(pub momo_core::Error);

impl From<momo_core::Error> for AppError {
    fn from(err: momo_core::Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            momo_core::Error::Config(_) | momo_core::Error::Json(_) => {
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            momo_core::Error::DeviceNotFound(_) => {
                (StatusCode::NOT_FOUND, self.0.to_string())
            }
            momo_core::Error::Pipeline(_) => {
                (StatusCode::CONFLICT, self.0.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
