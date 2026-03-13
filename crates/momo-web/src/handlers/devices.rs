use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceInfo {
    pub index: u32,
    pub name: String,
    pub status: String,
}

pub async fn get_devices() -> Json<Vec<DeviceInfo>> {
    // DeckLink enumeration returns empty in mock/no-hardware mode
    Json(Vec::new())
}
