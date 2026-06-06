use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::Json,
    response::Response,
    routing::get,
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;
use tokio::net::TcpListener;
use jsonwebtoken::{decode, Validation, DecodingKey, Algorithm};

use patient_management::patient_routes;
use patient_management::{AppState, Config};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check endpoint
    let path = request.uri().path();
    if path == "/health" {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["health_os"]);
    validation.set_audience(&["health_os_api"]);

    let token_data = decode::<serde_json::Value>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &validation,
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Extract user_id from token
    let user_id: String = token_data.claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();

    // Add user_id to request extensions
    request.extensions_mut().insert(user_id.to_string());

    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string()),
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string()),
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
                .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
                .layer(middleware::from_fn_with_state(AppState { db: db.clone(), config: config.clone() }, auth_middleware)),
        )
        .with_state(AppState { db, config: config.clone() });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Patient Management Service starting on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "patient-management",
        "version": "1.0.0"
    }))
}
