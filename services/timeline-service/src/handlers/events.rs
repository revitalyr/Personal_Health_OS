use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

pub async fn create_event(
    State(app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/patients/{}/events", patient_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // TODO: Validate event structure and convert to MedicalEvent
    // For now, create a mock event

    let event_id = Uuid::new_v4();

    // TODO: Store event in database
    // let event = MedicalEvent::new(...);
    // app.event_store.store_event(&event).await?;

    // TODO: Publish event to NATS
    // app.nats_client.publish_event(&event).await?;

    let response = serde_json::json!({
        "id": event_id,
        "patient_id": patient_id,
        "event_type": payload.get("event_type").unwrap_or(&Value::String("Unknown".to_string())),
        "timestamp": chrono::Utc::now(),
        "payload": payload,
        "status": "created",
        "message": "Event created successfully"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

pub async fn get_event(
    State(app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path((patient_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/events/{}", patient_id, event_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Get event from storage
    let event = app.event_store.get_event_by_id(event_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match event {
        Some(event) => {
            let response = serde_json::json!({
                "id": event.id,
                "patient_id": event.patient_id,
                "event_type": format!("{:?}", event.event_type),
                "timestamp": event.timestamp,
                "payload": event.payload,
                "metadata": event.metadata,
            });

            trace_response!(StatusCode::OK, std::time::Duration::from_millis(10));
            Ok(Json(response))
        }
        None => {
            trace_response!(StatusCode::NOT_FOUND, std::time::Duration::from_millis(5));
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn list_events(
    State(app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/events", patient_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Get events from storage
    let events = app.event_store.get_events_by_patient(patient_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = serde_json::json!({
        "patient_id": patient_id,
        "events": events.iter().map(|event| {
            serde_json::json!({
                "id": event.id,
                "event_type": format!("{:?}", event.event_type),
                "timestamp": event.timestamp,
                "payload": event.payload,
            })
        }).collect::<Vec<_>>(),
        "total_count": events.len(),
        "page": 1,
        "per_page": events.len(),
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(20));
    Ok(Json(response))
}
