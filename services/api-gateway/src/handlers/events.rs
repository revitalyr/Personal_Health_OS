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

    // TODO: Validate and store event
    let event_response = serde_json::json!({
        "id": Uuid::new_v4(),
        "patient_id": patient_id,
        "event_type": payload.get("event_type").unwrap_or(&Value::String("Unknown".to_string())),
        "timestamp": chrono::Utc::now(),
        "payload": payload,
        "status": "created"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(event_response))
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

    // TODO: Fetch event from storage
    let event = serde_json::json!({
        "id": event_id,
        "patient_id": patient_id,
        "event_type": "SymptomCreated",
        "timestamp": "2024-01-15T10:00:00Z",
        "payload": {
            "name": "Headache",
            "severity": 5
        }
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(15));
    Ok(Json(event))
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

    // TODO: Fetch events from storage
    let events = serde_json::json!({
        "patient_id": patient_id,
        "events": [],
        "total_count": 0,
        "page": 1,
        "per_page": 20
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(30));
    Ok(Json(events))
}
