use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
};
use event_model::{MedicalEvent, EventType};
use serde_json::Value;
use uuid::Uuid;

use telemetry::{trace_request, trace_response};

use crate::app::App;

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

    // Validate event structure and convert to MedicalEvent
    let event_type_str = payload.get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let event_type = match event_type_str {
        "SymptomCreated" => EventType::SymptomCreated,
        "MedicationStarted" => EventType::MedicationStarted,
        "MedicationStopped" => EventType::MedicationStopped,
        "LabResultReceived" => EventType::LabResultReceived,
        "DoctorVisit" => EventType::DoctorVisit,
        "Diagnosis" => EventType::Diagnosis,
        "DocumentUploaded" => EventType::DocumentUploaded,
        "ReminderTriggered" => EventType::ReminderTriggered,
        _ => EventType::SymptomCreated, // Default fallback
    };

    let event = MedicalEvent::new(
        patient_id,
        event_type,
        payload,
        "api_gateway".to_string(),
    );

    // Create event via service (store + publish to NATS)
    app.timeline_service.create_event(&event)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Publish timeline update notification
    app.nats_client.publish_timeline_update(patient_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to publish timeline update: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = serde_json::json!({
        "id": event.id,
        "patient_id": patient_id,
        "event_type": format!("{:?}", event.event_type),
        "timestamp": event.timestamp,
        "payload": event.payload,
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

    // Get event via service
    let event = app.timeline_service.get_event(event_id)
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

    // Get events via service
    let events = app.timeline_service.get_timeline(patient_id)
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
