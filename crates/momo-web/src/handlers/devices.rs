use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub index: u32,
    pub name: String,
    pub model_name: String,
    pub has_input: bool,
    pub has_output: bool,
    pub status: String,
}

pub async fn get_devices() -> Json<Vec<DeviceInfo>> {
    let mut infos: Vec<DeviceInfo> = momo_decklink::enumerate_devices()
        .into_iter()
        .map(|d| DeviceInfo {
            device_type: "DeckLink".to_string(),
            index: d.index,
            name: d.name,
            model_name: d.model_name,
            has_input: d.has_input,
            has_output: d.has_output,
            status: format!("{:?}", d.status),
        })
        .collect();

    let uvc_devices: Vec<DeviceInfo> = momo_uvc::enumerate_devices()
        .into_iter()
        .map(|d| DeviceInfo {
            device_type: "Uvc".to_string(),
            index: d.index,
            name: d.name.clone(),
            model_name: d.name,
            has_input: true,
            has_output: false,
            status: "Available".to_string(),
        })
        .collect();
    infos.extend(uvc_devices);

    Json(infos)
}
