use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use telemetry::trace_request;

use crate::app::App;

/// POST /patients/{patient_id}/events — Create a new health event.
pub async fn create_event(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/patients/{}/events", patient_id));

    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // API Gateway should proxy to timeline-service for actual event storage
    // For now, return error indicating feature requires timeline-service integration
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// GET /patients/{patient_id}/events/{event_id} — Retrieve a specific event.
pub async fn get_event(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path((patient_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/events/{}", patient_id, event_id));

    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // API Gateway should proxy to timeline-service for actual event retrieval
    // For now, return error indicating feature requires timeline-service integration
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// GET /patients/{patient_id}/events — List all events for a patient.
pub async fn list_events(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/events", patient_id));

    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // API Gateway should proxy to timeline-service for actual event retrieval
    // For now, return error indicating feature requires timeline-service integration
    Err(StatusCode::NOT_IMPLEMENTED)
}
