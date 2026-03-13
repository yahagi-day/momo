use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{Error, Result};
use crate::types::{DisplayMode, OutputTransform, PixelFormat};

/// Input source selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputSource {
    /// DeckLink input device.
    DeckLink {
        /// Device index (0-based).
        device_index: u32,
        /// Display mode to capture.
        display_mode: DisplayMode,
        /// Pixel format.
        pixel_format: PixelFormat,
    },
    /// UVC (USB camera) input.
    Uvc {
        /// Device path or identifier.
        device_path: String,
    },
}

/// Configuration for a single output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Unique output identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// DeckLink device index for this output.
    pub device_index: u32,
    /// Output display mode.
    pub display_mode: DisplayMode,
    /// Output pixel format.
    pub pixel_format: PixelFormat,
    /// Transform pipeline: crop → scale → flip.
    #[serde(default)]
    pub transform: OutputTransform,
    /// Whether this output is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Preview configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    /// Preview width.
    #[serde(default = "default_preview_width")]
    pub width: u32,
    /// Preview height.
    #[serde(default = "default_preview_height")]
    pub height: u32,
    /// Preview frame rate.
    #[serde(default = "default_preview_fps")]
    pub fps: u32,
    /// JPEG quality (1-100).
    #[serde(default = "default_jpeg_quality")]
    pub jpeg_quality: u8,
}

fn default_preview_width() -> u32 {
    640
}
fn default_preview_height() -> u32 {
    360
}
fn default_preview_fps() -> u32 {
    10
}
fn default_jpeg_quality() -> u8 {
    75
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            width: default_preview_width(),
            height: default_preview_height(),
            fps: default_preview_fps(),
            jpeg_quality: default_jpeg_quality(),
        }
    }
}

/// Web server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    /// Listen address.
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    /// Listen port.
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8080
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            port: default_port(),
        }
    }
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Input source.
    pub input: InputSource,
    /// Output configurations.
    pub outputs: Vec<OutputConfig>,
    /// Preview settings.
    #[serde(default)]
    pub preview: PreviewConfig,
    /// Web server settings.
    #[serde(default)]
    pub web: WebConfig,
}

impl Config {
    /// Load configuration from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Parse configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        let config: Config = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// Serialize configuration to a JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.outputs.is_empty() {
            return Err(Error::Config("at least one output is required".into()));
        }

        // Check for duplicate output IDs
        let mut ids = std::collections::HashSet::new();
        for output in &self.outputs {
            if !ids.insert(&output.id) {
                return Err(Error::Config(format!(
                    "duplicate output id: {}",
                    output.id
                )));
            }
        }

        // Validate crop regions
        for output in &self.outputs {
            if let Some(crop) = &output.transform.crop {
                if crop.width == 0 || crop.height == 0 {
                    return Err(Error::Config(format!(
                        "output '{}': crop region must have non-zero dimensions",
                        output.id
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> Config {
        Config {
            input: InputSource::DeckLink {
                device_index: 0,
                display_mode: DisplayMode::Hd1080p5994,
                pixel_format: PixelFormat::Uyvy,
            },
            outputs: vec![
                OutputConfig {
                    id: "out1".into(),
                    name: "Main Output".into(),
                    device_index: 1,
                    display_mode: DisplayMode::Hd1080p5994,
                    pixel_format: PixelFormat::Uyvy,
                    transform: OutputTransform::default(),
                    enabled: true,
                },
                OutputConfig {
                    id: "out2".into(),
                    name: "Secondary Output".into(),
                    device_index: 2,
                    display_mode: DisplayMode::Hd1080p5994,
                    pixel_format: PixelFormat::Uyvy,
                    transform: OutputTransform {
                        crop: Some(crate::types::CropRegion {
                            x: 0,
                            y: 0,
                            width: 960,
                            height: 1080,
                        }),
                        flip: crate::types::FlipOptions {
                            horizontal: true,
                            vertical: false,
                        },
                    },
                    enabled: true,
                },
            ],
            preview: PreviewConfig::default(),
            web: WebConfig::default(),
        }
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = sample_config();
        let json = config.to_json().expect("serialize");
        let restored = Config::from_json(&json).expect("deserialize");

        assert_eq!(restored.outputs.len(), 2);
        assert_eq!(restored.outputs[0].id, "out1");
        assert_eq!(restored.outputs[1].id, "out2");
        assert!(restored.outputs[1].transform.crop.is_some());
        assert!(restored.outputs[1].transform.flip.horizontal);
        assert!(!restored.outputs[1].transform.flip.vertical);
        assert_eq!(restored.preview.width, 640);
        assert_eq!(restored.preview.height, 360);
        assert_eq!(restored.web.port, 8080);
    }

    #[test]
    fn config_file_roundtrip() {
        let config = sample_config();
        let dir = std::env::temp_dir().join("momo-test-config");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test-config.json");

        config.save(&path).expect("save");
        let restored = Config::load(&path).expect("load");

        assert_eq!(restored.outputs.len(), 2);
        assert_eq!(restored.outputs[0].name, "Main Output");

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_rejects_empty_outputs() {
        let json = r#"{
            "input": { "type": "DeckLink", "device_index": 0, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" },
            "outputs": []
        }"#;
        let err = Config::from_json(json).unwrap_err();
        assert!(err.to_string().contains("at least one output"));
    }

    #[test]
    fn config_rejects_duplicate_ids() {
        let json = r#"{
            "input": { "type": "DeckLink", "device_index": 0, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" },
            "outputs": [
                { "id": "a", "name": "A", "device_index": 1, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" },
                { "id": "a", "name": "B", "device_index": 2, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" }
            ]
        }"#;
        let err = Config::from_json(json).unwrap_err();
        assert!(err.to_string().contains("duplicate output id"));
    }

    #[test]
    fn config_rejects_zero_crop() {
        let json = r#"{
            "input": { "type": "DeckLink", "device_index": 0, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" },
            "outputs": [
                {
                    "id": "a", "name": "A", "device_index": 1,
                    "display_mode": "Hd1080p5994", "pixel_format": "Uyvy",
                    "transform": { "crop": { "x": 0, "y": 0, "width": 0, "height": 100 } }
                }
            ]
        }"#;
        let err = Config::from_json(json).unwrap_err();
        assert!(err.to_string().contains("non-zero dimensions"));
    }

    #[test]
    fn config_defaults_applied() {
        let json = r#"{
            "input": { "type": "DeckLink", "device_index": 0, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" },
            "outputs": [
                { "id": "a", "name": "A", "device_index": 1, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" }
            ]
        }"#;
        let config = Config::from_json(json).expect("parse");
        assert!(config.outputs[0].enabled);
        assert!(config.outputs[0].transform.crop.is_none());
        assert!(!config.outputs[0].transform.flip.horizontal);
        assert_eq!(config.preview.fps, 10);
        assert_eq!(config.web.bind_address, "0.0.0.0");
    }

    #[test]
    fn input_source_uvc_roundtrip() {
        let json = r#"{
            "input": { "type": "Uvc", "device_path": "/dev/video0" },
            "outputs": [
                { "id": "a", "name": "A", "device_index": 1, "display_mode": "Hd1080p5994", "pixel_format": "Uyvy" }
            ]
        }"#;
        let config = Config::from_json(json).expect("parse");
        match &config.input {
            InputSource::Uvc { device_path } => assert_eq!(device_path, "/dev/video0"),
            _ => panic!("expected UVC input"),
        }
    }
}
