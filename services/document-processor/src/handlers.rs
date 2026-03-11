pub mod documents;
pub mod manual_input;
pub mod dicom;
pub mod ocr;

use axum::Json;
use serde_json::Value;

pub async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "document-processor",
        "timestamp": chrono::Utc::now(),
        "features": {
            "ocr": true,
            "dicom": true,
            "manual_input": true,
            "batch_processing": true
        }
    }))
}
