use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, error};

use patient_management::patient_routes;
use telemetry;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    telemetry::init();

    // Load configuration
    let config = Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string()),
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key".to_string()),
        port: std::env::var("PORT")
            .unwrap_or_else(|_| "8085".to_string())
            .parse()
            .unwrap_or(8085),
    };

    // Initialize database
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    info!("Connected to database successfully");

    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", patient_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)),
        )
        .with_state(AppState { db, config: config.clone() });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Patient Management Service starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "patient-management",
        "version": "1.0.0"
    }))
}
