use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("DeckLink error: {0}")]
    DeckLink(String),

    #[error("UVC error: {0}")]
    Uvc(String),

    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device disconnected: {0}")]
    DeviceDisconnected(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
