use std::path::Path;

use clap::Parser;
use momo_core::config::{
    Config, InputSource, OutputConfig, PreviewConfig, WebConfig,
};
use momo_core::types::{DisplayMode, OutputTransform, PixelFormat};
use momo_pipeline::Pipeline;
use momo_web::state::AppState;

// ANSI color helpers
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const PINK: &str = "\x1b[38;5;205m";
const GRAY: &str = "\x1b[38;5;244m";

fn print_banner(addr: &str) {
    let version = env!("CARGO_PKG_VERSION");

    let decklink = if cfg!(feature = "decklink") { "on" } else { "off" };
    let gpu = if cfg!(feature = "gpu") { "on" } else { "off" };
    let webrtc = if cfg!(feature = "webrtc") { "on" } else { "off" };

    println!("\n\
  {BOLD}{PINK}  ███╗   ███╗  ██████╗  ███╗   ███╗  ██████╗{RESET}\n\
  {BOLD}{PINK}  ████╗ ████║ ██╔═══██╗ ████╗ ████║ ██╔═══██╗{RESET}\n\
  {BOLD}{PINK}  ██╔████╔██║ ██║   ██║ ██╔████╔██║ ██║   ██║{RESET}\n\
  {BOLD}{PINK}  ██║╚██╔╝██║ ██║   ██║ ██║╚██╔╝██║ ██║   ██║{RESET}\n\
  {BOLD}{PINK}  ██║ ╚═╝ ██║ ╚██████╔╝ ██║ ╚═╝ ██║ ╚██████╔╝{RESET}\n\
  {BOLD}{PINK}  ╚═╝     ╚═╝  ╚═════╝  ╚═╝     ╚═╝  ╚═════╝{RESET}\n\
  {GRAY}v{version} — live video splitter{RESET}\n\
\n\
  {PINK}◆{RESET} DeckLink: {decklink}  {PINK}◆{RESET} GPU: {gpu}  {PINK}◆{RESET} WebRTC: {webrtc}\n\
  {PINK}◆{RESET} Server: {addr}\n\
    ");
}

/// momo — live video splitter/router
#[derive(Parser, Debug)]
#[command(name = "momo", version, about)]
struct Args {
    /// Path to configuration JSON file.
    #[arg(short, long, default_value = "config.json")]
    config: String,

    /// Web server bind address.
    #[arg(long, default_value = "0.0.0.0")]
    bind: String,

    /// Web server port.
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

fn default_config() -> Config {
    Config {
        input: InputSource::Mock {
            width: 1920,
            height: 1080,
            fps: 30,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let addr = format!("{}:{}", args.bind, args.port);
    print_banner(&addr);

    let mut pipeline = Pipeline::new();

    let config_path = Path::new(&args.config);
    if config_path.exists() {
        match pipeline.load_config(config_path) {
            Ok(()) => tracing::info!("loaded config from {}", args.config),
            Err(e) => {
                tracing::error!("failed to load config: {e}");
                std::process::exit(1);
            }
        }
    } else {
        tracing::warn!(
            "config file '{}' not found, using default mock config",
            args.config
        );
        let config = default_config();
        pipeline.set_config(config).expect("default config is valid");
        pipeline.set_config_path(config_path.to_path_buf());
    }

    let state = AppState::new(pipeline);
    let app = momo_web::build_router(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
