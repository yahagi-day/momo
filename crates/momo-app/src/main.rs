use clap::Parser;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    tracing::info!("momo starting with config: {}", args.config);

    let _pipeline = momo_pipeline::Pipeline::new();

    let app = momo_web::build_router();
    let addr = format!("{}:{}", args.bind, args.port);
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
