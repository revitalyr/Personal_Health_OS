use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{app::App, trace_request, trace_response};

pub async fn generate_access(
    State(app): State<App>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", format!("/doctor-access/{}", patient_id));
    
    // Generate doctor access token (15 minutes validity)
    let token = app.auth_service.generate_doctor_access_token(patient_id, 15)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate QR code for the token
    let qr_code_data = format!("https://healthos.app/doctor-view/{}", token);
    
    // TODO: Generate actual QR code image
    let response = serde_json::json!({
        "patient_id": patient_id,
        "access_token": token,
        "qr_code": qr_code_data,
        "expires_at": chrono::Utc::now() + chrono::Duration::minutes(15),
        "access_url": format!("https://healthos.app/doctor-view/{}", token)
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(25));
    Ok(Json(response))
}

pub async fn view_report(
    State(app): State<App>,
    Path(token): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("GET", format!("/doctor-view/{}", token));
    
    // Validate doctor access token
    let patient_id = app.auth_service.validate_doctor_access_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // TODO: Generate and return doctor report
    let report = serde_json::json!({
        "patient_id": patient_id,
        "generated_at": chrono::Utc::now(),
        "valid_until": chrono::Utc::now() + chrono::Duration::minutes(15),
        "summary": {
            "total_events": 5,
            "recent_symptoms": ["Headache", "Fatigue"],
            "current_medications": ["Ibuprofen 400mg"],
            "recent_visits": [
                {
                    "date": "2024-01-10",
                    "doctor": "Dr. Smith",
                    "specialty": "General Practice",
                    "reason": "Regular checkup"
                }
            ]
        },
        "timeline": [
            {
                "date": "2024-01-15",
                "events": [
                    {
                        "time": "10:00",
                        "type": "Symptom",
                        "description": "Headache (moderate)"
                    },
                    {
                        "time": "12:00", 
                        "type": "Medication",
                        "description": "Ibuprofen 400mg taken"
                    }
                ]
            }
        ],
        "recommendations": [
            "Monitor headache frequency",
            "Consider follow-up if symptoms persist",
            "Maintain current medication regimen"
        ]
    });

    trace_response!(StatusCode::OK, std::time::Duration::from_millis(40));
    Ok(Json(report))
}
