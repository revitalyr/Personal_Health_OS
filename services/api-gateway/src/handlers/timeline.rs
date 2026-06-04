use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

pub async fn get_timeline(
    State(_app): State<App>,
    Extension(user_id): Extension<Uuid>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/patients/{}/timeline", patient_id));

    // Authorization: verify user_id matches patient_id
    if user_id != patient_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // TODO: Call timeline service
    // For now, return mock data
    let timeline = serde_json::json!({
        "patient_id": patient_id,
        "events": [
            {
                "id": Uuid::new_v4(),
                "event_type": "SymptomCreated",
                "timestamp": "2024-01-15T10:00:00Z",
                "payload": {
                    "name": "Headache",
                    "severity": 5
                }
            },
            {
                "id": Uuid::new_v4(),
                "event_type": "MedicationStarted",
                "timestamp": "2024-01-15T12:00:00Z",
                "payload": {
                    "name": "Ibuprofen",
                    "dosage": "400mg"
                }
            }
        ],
        "summary": {
            "total_events": 2,
            "date_range": {
                "start": "2024-01-15T10:00:00Z",
                "end": "2024-01-15T12:00:00Z"
            }
        }
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(25));
    Ok(Json(timeline))
}
