use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use serde_json::Value;

mod app;
mod config;
mod handlers;
mod routes;

use app::App;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry
    telemetry::init_telemetry("doctor-access-service")?;

    // Load configuration
    let config = Config::from_env()?;
    info!("Doctor Access Service configuration loaded");

    // Build application
    let app = App::build(&config).await?;

    // Create router
    let router = routes::create_router(app);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Doctor Access Service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Shutdown telemetry
    telemetry::shutdown();
    info!("Doctor Access Service shutdown complete");

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
