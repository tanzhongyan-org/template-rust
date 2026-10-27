use std::net::SocketAddr;
use template_rust::api::app_router;
use template_rust::core::config::AppConfig;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = app_router();
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        "Application {} [{}] listening on http://{}",
        config.app_name, config.app_env, addr
    );

    axum::serve(listener, app).await?;

    Ok(())
}
