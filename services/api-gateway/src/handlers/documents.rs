use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

pub async fn upload_document(
    State(_app): State<App>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/documents/upload");
    
    // TODO: Process uploaded document
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("unknown");
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        
        tracing::info!(
            "Received file: {} (name: {}, type: {})",
            filename,
            name,
            content_type
        );
        
        // TODO: Store file and process it
    }

    let response = serde_json::json!({
        "id": Uuid::new_v4(),
        "status": "uploaded",
        "message": "Document uploaded successfully"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(100));
    Ok(Json(response))
}
