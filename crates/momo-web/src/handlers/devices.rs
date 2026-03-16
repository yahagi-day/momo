use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceInfo {
    pub index: u32,
    pub name: String,
    pub model_name: String,
    pub has_input: bool,
    pub has_output: bool,
    pub status: String,
}

pub async fn get_devices() -> Json<Vec<DeviceInfo>> {
    let devices = momo_decklink::enumerate_devices();
    let infos = devices
        .into_iter()
        .map(|d| DeviceInfo {
            index: d.index,
            name: d.name,
            model_name: d.model_name,
            has_input: d.has_input,
            has_output: d.has_output,
            status: format!("{:?}", d.status),
        })
        .collect();
    Json(infos)
}
