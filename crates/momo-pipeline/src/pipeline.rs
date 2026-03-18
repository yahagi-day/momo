use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use momo_core::config::{Config, OutputConfig};
use momo_core::error::{Error, Result};
use momo_core::types::{OutputTransform, PipelineState};
use tokio::sync::broadcast;

use momo_gpu::GpuProcessor;

use crate::event::PipelineEvent;
use crate::input::InputDriver;
use crate::preview::encode_preview;

pub struct Pipeline {
    state: PipelineState,
    config: Option<Config>,
    config_path: Option<PathBuf>,
    event_tx: broadcast::Sender<PipelineEvent>,
    preview_tx: broadcast::Sender<Vec<u8>>,
    output_preview_txs: HashMap<String, broadcast::Sender<Vec<u8>>>,
    running: Option<RunningState>,
}

struct RunningState {
    stop_flag: Arc<AtomicBool>,
    _input_thread: std::thread::JoinHandle<()>,
    bridge_task: tokio::task::JoinHandle<()>,
    preview_task: tokio::task::JoinHandle<()>,
    output_configs_tx: tokio::sync::watch::Sender<Vec<OutputConfig>>,
}

impl Pipeline {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(64);
        let (preview_tx, _) = broadcast::channel(4);
        Self {
            state: PipelineState::Stopped,
            config: None,
            config_path: None,
            event_tx,
            preview_tx,
            output_preview_txs: HashMap::new(),
            running: None,
        }
    }

    pub fn state(&self) -> PipelineState {
        self.state
    }

    pub fn config(&self) -> Option<&Config> {
        self.config.as_ref()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PipelineEvent> {
        self.event_tx.subscribe()
    }

    pub fn subscribe_preview(&self) -> broadcast::Receiver<Vec<u8>> {
        self.preview_tx.subscribe()
    }

    pub fn subscribe_output_preview(&self, id: &str) -> Option<broadcast::Receiver<Vec<u8>>> {
        self.output_preview_txs.get(id).map(|tx| tx.subscribe())
    }

    pub fn set_config(&mut self, config: Config) -> Result<()> {
        config.validate()?;
        self.config = Some(config);
        self.emit(PipelineEvent::ConfigChanged);
        Ok(())
    }

    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = Some(path);
    }

    pub fn start(&mut self) -> Result<()> {
        if self.state != PipelineState::Stopped {
            return Err(Error::Pipeline("pipeline is not stopped".into()));
        }

        let config = self
            .config
            .as_ref()
            .ok_or_else(|| Error::Config("no configuration set".into()))?
            .clone();

        self.state = PipelineState::Starting;
        self.emit(PipelineEvent::StateChanged { state: self.state });

        let driver = InputDriver::from_config(&config.input)?;

        let (frame_tx, frame_rx) = crossbeam_channel::bounded(4);
        let stop_flag = Arc::new(AtomicBool::new(false));

        let input_thread = driver.start(frame_tx, stop_flag.clone());

        // Bridge: crossbeam → tokio mpsc
        let (bridge_tx, mut bridge_rx) = tokio::sync::mpsc::channel(2);
        let bridge_stop = stop_flag.clone();
        let bridge_task = tokio::task::spawn_blocking(move || {
            while !bridge_stop.load(Ordering::Relaxed) {
                match frame_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(frame) => {
                        if bridge_tx.blocking_send(frame).is_err() {
                            break;
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        // Collect enabled output configs, shared via watch channel for live updates
        let output_configs: Vec<_> = config
            .outputs
            .iter()
            .filter(|o| o.enabled)
            .cloned()
            .collect();
        let (output_configs_tx, output_configs_rx) =
            tokio::sync::watch::channel(output_configs.clone());

        // Create per-output preview channels
        let mut output_preview_txs_map = HashMap::new();
        let mut output_preview_txs_task = HashMap::new();
        for output in &output_configs {
            let (tx, _) = broadcast::channel(4);
            output_preview_txs_task.insert(output.id.clone(), tx.clone());
            output_preview_txs_map.insert(output.id.clone(), tx);
        }
        self.output_preview_txs = output_preview_txs_map;

        // Create and start DeckLink outputs (feature-gated)
        #[cfg(feature = "decklink")]
        let mut output_players: HashMap<String, momo_decklink::output::DeckLinkOutput> = {
            let mut players = HashMap::new();
            for output in &output_configs {
                let mut player = momo_decklink::output::DeckLinkOutput::new(
                    output.device_index,
                    output.display_mode,
                    output.pixel_format,
                );
                match momo_decklink::VideoOutput::start(&mut player) {
                    Ok(()) => {
                        tracing::info!(output_id = %output.id, "DeckLink output started");
                        players.insert(output.id.clone(), player);
                    }
                    Err(e) => {
                        tracing::warn!(
                            output_id = %output.id,
                            "Failed to start DeckLink output: {e}"
                        );
                    }
                }
            }
            players
        };

        // Preview encoding + FPS tracking + per-output GPU transforms + DeckLink output
        let preview_tx = self.preview_tx.clone();
        let event_tx = self.event_tx.clone();
        let preview_config = config.preview.clone();
        let preview_task = tokio::spawn(async move {
            let mut frame_count = 0u64;
            let mut last_fps_time = tokio::time::Instant::now();
            let gpu = GpuProcessor::new();
            let preview_interval = Duration::from_secs_f64(1.0 / preview_config.fps as f64);
            let mut last_output_preview_time = tokio::time::Instant::now()
                .checked_sub(preview_interval)
                .unwrap_or_else(tokio::time::Instant::now);

            while let Some(frame) = bridge_rx.recv().await {
                frame_count += 1;
                let elapsed = last_fps_time.elapsed();
                if elapsed >= Duration::from_secs(1) {
                    let fps = frame_count as f64 / elapsed.as_secs_f64();
                    frame_count = 0;
                    last_fps_time = tokio::time::Instant::now();
                    let _ = event_tx.send(PipelineEvent::FpsUpdate { fps });
                }

                let now = tokio::time::Instant::now();
                let should_encode_output_preview =
                    now.duration_since(last_output_preview_time) >= preview_interval;

                // Read latest output configs (updated via watch channel on crop/transform changes)
                let current_outputs = output_configs_rx.borrow().clone();

                // Per-output GPU transform + DeckLink output
                for output in &current_outputs {
                    let output_resolution = output.display_mode.resolution();
                    match gpu.process(&frame, &output.transform, output_resolution) {
                        Ok(transformed) => {
                            // Send transformed frame to DeckLink output device
                            #[cfg(feature = "decklink")]
                            {
                                if let Some(player) = output_players.get_mut(&output.id) {
                                    if let Err(e) =
                                        momo_decklink::VideoOutput::send_frame(player, &transformed)
                                    {
                                        tracing::warn!(
                                            output_id = %output.id,
                                            "DeckLink output error: {e}"
                                        );
                                    }
                                }
                            }

                            tracing::trace!(
                                output_id = %output.id,
                                "transformed frame ready ({}x{})",
                                output_resolution.width,
                                output_resolution.height,
                            );

                            // Encode output preview if throttle allows and someone is listening
                            if should_encode_output_preview {
                                if let Some(tx) = output_preview_txs_task.get(&output.id) {
                                    if tx.receiver_count() > 0 {
                                        let pc = preview_config.clone();
                                        if let Ok(Ok(jpeg)) =
                                            tokio::task::spawn_blocking(move || {
                                                encode_preview(&transformed, &pc)
                                            })
                                            .await
                                        {
                                            let _ = tx.send(jpeg);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                output_id = %output.id,
                                "GPU transform failed: {e}"
                            );
                        }
                    }
                }

                if should_encode_output_preview {
                    last_output_preview_time = now;
                }

                // Preview encode
                let pc = preview_config.clone();
                if let Ok(Ok(jpeg)) =
                    tokio::task::spawn_blocking(move || encode_preview(&frame, &pc)).await
                {
                    let _ = preview_tx.send(jpeg);
                }
            }

            // Graceful shutdown: stop DeckLink outputs
            #[cfg(feature = "decklink")]
            for (_id, player) in output_players.iter_mut() {
                let _ = momo_decklink::VideoOutput::stop(player);
            }
        });

        self.running = Some(RunningState {
            stop_flag,
            _input_thread: input_thread,
            bridge_task,
            preview_task,
            output_configs_tx,
        });

        self.state = PipelineState::Running;
        self.emit(PipelineEvent::StateChanged { state: self.state });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if self.state != PipelineState::Running {
            return Err(Error::Pipeline("pipeline is not running".into()));
        }

        self.state = PipelineState::Stopping;
        self.emit(PipelineEvent::StateChanged { state: self.state });

        if let Some(running) = self.running.take() {
            running.stop_flag.store(true, Ordering::Relaxed);
            running.preview_task.abort();
            running.bridge_task.abort();
        }

        self.output_preview_txs.clear();
        self.state = PipelineState::Stopped;
        self.emit(PipelineEvent::StateChanged { state: self.state });

        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| Error::Config("no configuration to save".into()))?;
        let path = self
            .config_path
            .as_ref()
            .ok_or_else(|| Error::Config("no config path set".into()))?;
        config.save(path)
    }

    pub fn load_config(&mut self, path: &Path) -> Result<()> {
        let config = Config::load(path)?;
        self.config = Some(config);
        self.config_path = Some(path.to_path_buf());
        self.emit(PipelineEvent::ConfigChanged);
        Ok(())
    }

    pub fn update_output(&mut self, id: &str, transform: OutputTransform) -> Result<()> {
        let config = self
            .config
            .as_mut()
            .ok_or_else(|| Error::Config("no configuration set".into()))?;
        let output = config
            .outputs
            .iter_mut()
            .find(|o| o.id == id)
            .ok_or_else(|| Error::DeviceNotFound(format!("output '{id}' not found")))?;
        output.transform = transform;

        // Propagate to running frame loop via watch channel
        if let Some(ref running) = self.running {
            let updated: Vec<_> = config
                .outputs
                .iter()
                .filter(|o| o.enabled)
                .cloned()
                .collect();
            let _ = running.output_configs_tx.send(updated);
        }

        self.emit(PipelineEvent::ConfigChanged);
        Ok(())
    }

    pub fn outputs(&self) -> &[OutputConfig] {
        self.config
            .as_ref()
            .map(|c| c.outputs.as_slice())
            .unwrap_or(&[])
    }

    fn emit(&self, event: PipelineEvent) {
        let _ = self.event_tx.send(event);
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use momo_core::config::{InputSource, OutputConfig, PreviewConfig, WebConfig};
    use momo_core::types::{DisplayMode, FlipOptions, PixelFormat};

    fn mock_config() -> Config {
        Config {
            input: InputSource::Mock {
                width: 320,
                height: 240,
                fps: 10,
            },
            outputs: vec![OutputConfig {
                id: "out1".into(),
                name: "Output 1".into(),
                device_index: 0,
                display_mode: DisplayMode::Hd1080p5994,
                pixel_format: PixelFormat::Uyvy,
                transform: OutputTransform::default(),
                enabled: true,
            }],
            preview: PreviewConfig::default(),
            web: WebConfig::default(),
        }
    }

    #[test]
    fn initial_state_is_stopped() {
        let pipeline = Pipeline::new();
        assert_eq!(pipeline.state(), PipelineState::Stopped);
        assert!(pipeline.config().is_none());
    }

    #[test]
    fn set_config() {
        let mut pipeline = Pipeline::new();
        let mut rx = pipeline.subscribe();

        pipeline.set_config(mock_config()).unwrap();
        assert!(pipeline.config().is_some());

        let event = rx.try_recv().unwrap();
        assert!(matches!(event, PipelineEvent::ConfigChanged));
    }

    #[test]
    fn update_output_transform() {
        let mut pipeline = Pipeline::new();
        pipeline.set_config(mock_config()).unwrap();

        let transform = OutputTransform {
            crop: None,
            flip: FlipOptions {
                horizontal: true,
                vertical: false,
            },
        };
        pipeline.update_output("out1", transform).unwrap();

        let output = &pipeline.config().unwrap().outputs[0];
        assert!(output.transform.flip.horizontal);
    }

    #[test]
    fn update_output_not_found() {
        let mut pipeline = Pipeline::new();
        pipeline.set_config(mock_config()).unwrap();

        let result = pipeline.update_output("nonexistent", OutputTransform::default());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn start_stop_lifecycle() {
        let mut pipeline = Pipeline::new();
        pipeline.set_config(mock_config()).unwrap();

        let mut rx = pipeline.subscribe();

        pipeline.start().unwrap();
        assert_eq!(pipeline.state(), PipelineState::Running);

        // Drain events
        while rx.try_recv().is_ok() {}

        pipeline.stop().unwrap();
        assert_eq!(pipeline.state(), PipelineState::Stopped);
    }

    #[test]
    fn start_without_config_fails() {
        let mut pipeline = Pipeline::new();
        assert!(pipeline.start().is_err());
    }

    #[test]
    fn stop_when_stopped_fails() {
        let mut pipeline = Pipeline::new();
        assert!(pipeline.stop().is_err());
    }

    #[tokio::test]
    async fn subscribe_output_preview_lifecycle() {
        let mut pipeline = Pipeline::new();
        pipeline.set_config(mock_config()).unwrap();

        // Before start: None
        assert!(pipeline.subscribe_output_preview("out1").is_none());

        pipeline.start().unwrap();

        // After start: Some
        assert!(pipeline.subscribe_output_preview("out1").is_some());
        // Unknown id: None
        assert!(pipeline.subscribe_output_preview("nonexistent").is_none());

        pipeline.stop().unwrap();

        // After stop: None
        assert!(pipeline.subscribe_output_preview("out1").is_none());
    }

    #[tokio::test]
    async fn subscribe_preview() {
        let mut pipeline = Pipeline::new();
        pipeline.set_config(mock_config()).unwrap();
        let mut preview_rx = pipeline.subscribe_preview();

        pipeline.start().unwrap();

        // Wait for at least one preview frame
        let result =
            tokio::time::timeout(Duration::from_secs(3), preview_rx.recv()).await;
        assert!(result.is_ok(), "should receive a preview frame");

        pipeline.stop().unwrap();
    }
}
