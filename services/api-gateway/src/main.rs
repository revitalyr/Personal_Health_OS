use std::net::SocketAddr;
use tracing::info;

mod app;
mod config;
mod handlers;
mod middleware;
mod routes;

use app::App;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry
    telemetry::init_telemetry("api-gateway")
        .map_err(|e| anyhow::anyhow!("Failed to initialize telemetry: {}", e))?;

    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    let app = App::build(&config).await?;

    let router = routes::create_router(app);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("API Gateway starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;


    telemetry::shutdown();
    info!("API Gateway shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
