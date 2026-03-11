pub mod auth;
pub mod timeline;
pub mod events;
pub mod documents;
pub mod ai;
pub mod doctor_access;

use axum::Json;
use serde_json::Value;

pub async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "api-gateway",
        "timestamp": chrono::Utc::now()
    }))
}
