use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

pub async fn create_event(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/patients/{}/events", patient_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // API Gateway should proxy to timeline-service for actual event storage
    // For now, return error indicating feature requires timeline-service integration
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn get_event(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path((patient_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/events/{}", patient_id, event_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // API Gateway should proxy to timeline-service for actual event retrieval
    // For now, return error indicating feature requires timeline-service integration
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn list_events(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/events", patient_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // API Gateway should proxy to timeline-service for actual event retrieval
    // For now, return error indicating feature requires timeline-service integration
    Err(StatusCode::NOT_IMPLEMENTED)
}
