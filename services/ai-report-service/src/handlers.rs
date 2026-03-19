use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::app::App;

pub async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "ai-report-service",
        "timestamp": chrono::Utc::now(),
        "features": {
            "ai_reports": true,
            "openai": false,
            "ollama": false
        }
    }))
}

pub async fn generate_report(
    State(_app): State<App>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Generating AI report for patient: {:?}", payload.get("patient_id"));
    
    let report_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "report_id": report_id,
        "status": "generated",
        "generated_at": chrono::Utc::now(),
        "summary": "AI-generated medical summary",
        "recommendations": [
            "Continue current treatment plan",
            "Schedule follow-up in 2 weeks",
            "Monitor symptoms closely"
        ]
    });

    Ok(Json(response))
}
