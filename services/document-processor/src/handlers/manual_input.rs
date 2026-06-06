use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::app::App;
use telemetry::{trace_request, trace_response};

// Manual input structures
#[derive(Debug, Deserialize)]
pub struct SymptomInput {
    pub patient_id: Uuid,
    pub name: String,
    pub severity: u8, // 1-10 scale
    pub description: Option<String>,
    pub duration: Option<String>,
    pub started_at: Option<chrono::DateTime<Utc>>,
    pub location: Option<String>, // body part
    pub triggers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct MedicationInput {
    pub patient_id: Uuid,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub route: Option<String>, // oral, injection, etc.
    pub start_date: chrono::DateTime<Utc>,
    pub end_date: Option<chrono::DateTime<Utc>>,
    pub prescribed_by: Option<String>,
    pub reason: Option<String>,
    pub side_effects: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct LabResultInput {
    pub patient_id: Uuid,
    pub test_name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: Option<String>,
    pub status: String, // normal, high, low, critical
    pub facility: String,
    pub test_date: NaiveDate,
    pub doctor: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DoctorVisitInput {
    pub patient_id: Uuid,
    pub doctor_name: String,
    pub specialty: String,
    pub facility: String,
    pub visit_date: chrono::DateTime<Utc>,
    pub reason: String,
    pub diagnosis: Option<Vec<String>>,
    pub symptoms_discussed: Option<Vec<String>>,
    pub medications_prescribed: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
    pub follow_up_date: Option<chrono::DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DiagnosisInput {
    pub patient_id: Uuid,
    pub condition: String,
    pub icd10_code: Option<String>,
    pub diagnosed_by: String,
    pub diagnosis_date: NaiveDate,
    pub severity: Option<String>, // mild, moderate, severe
    pub acute: bool,
    pub chronic: bool,
    pub notes: Option<String>,
    pub treatment_plan: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ManualEntryInput {
    pub patient_id: Uuid,
    pub entry_type: String,
    pub title: String,
    pub description: String,
    pub entry_date: chrono::DateTime<Utc>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub attachments: Option<Vec<Uuid>>, // Document IDs
    pub importance: Option<String>, // low, medium, high
}

// Symptom creation
pub async fn create_symptom(
    State(app): State<App>,
    Json(payload): Json<SymptomInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/symptom");
    
    // Validate severity
    if payload.severity < 1 || payload.severity > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Create symptom event
    let event_id = app.document_service.create_symptom_event(payload).await
        .map_err(|e| {
            tracing::error!("Failed to create symptom event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "symptom",
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

// Medication creation
pub async fn create_medication(
    State(app): State<App>,
    Json(payload): Json<MedicationInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/medication");
    
    // Validate dates
    if let Some(end_date) = payload.end_date {
        if end_date <= payload.start_date {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Create medication event
    let event_id = app.document_service.create_medication_event(payload).await
        .map_err(|e| {
            tracing::error!("Failed to create medication event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "medication",
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

// Lab result creation
pub async fn create_lab_result(
    State(app): State<App>,
    Json(payload): Json<LabResultInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/lab-result");
    
    // Validate status
    let valid_statuses = ["normal", "high", "low", "critical", "borderline"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Create lab result event
    let event_id = app.document_service.create_lab_result_event(payload).await
        .map_err(|e| {
            tracing::error!("Failed to create lab result event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "lab_result",
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

// Doctor visit creation
pub async fn create_doctor_visit(
    State(app): State<App>,
    Json(payload): Json<DoctorVisitInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/doctor-visit");
    
    // Create doctor visit event
    let event_id = app.document_service.create_doctor_visit_event(payload).await
        .map_err(|e| {
            tracing::error!("Failed to create doctor visit event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "doctor_visit",
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

// Diagnosis creation
pub async fn create_diagnosis(
    State(app): State<App>,
    Json(payload): Json<DiagnosisInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/diagnosis");
    
    // Validate severity
    if let Some(ref severity) = payload.severity {
        let valid_severities = ["mild", "moderate", "severe"];
        if !valid_severities.contains(&severity.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Create diagnosis event
    let event_id = app.document_service.create_diagnosis_event(payload).await
        .map_err(|e| {
            tracing::error!("Failed to create diagnosis event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "diagnosis",
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

// General manual entry
pub async fn create_manual_entry(
    State(app): State<App>,
    Json(payload): Json<ManualEntryInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/manual");
    
    // Validate importance
    if let Some(ref importance) = payload.importance {
        let valid_importance = ["low", "medium", "high"];
        if !valid_importance.contains(&importance.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Create manual entry event
    let event_id = app.document_service.create_manual_entry_event(payload).await
        .map_err(|e| {
            tracing::error!("Failed to create manual entry event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "manual_entry",
        "status": "created",
        "created_at": chrono::Utc::now()
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

// Helper structures for quick input
#[derive(Debug, Deserialize)]
pub struct QuickSymptomInput {
    pub patient_id: Uuid,
    pub name: String,
    pub severity: u8,
}

#[derive(Debug, Deserialize)]
pub struct QuickMedicationInput {
    pub patient_id: Uuid,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
}

// Quick input endpoints (simplified versions)
pub async fn create_quick_symptom(
    State(app): State<App>,
    Json(payload): Json<QuickSymptomInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/quick-symptom");
    
    let full_input = SymptomInput {
        patient_id: payload.patient_id,
        name: payload.name,
        severity: payload.severity,
        description: None,
        duration: None,
        started_at: Some(Utc::now()),
        location: None,
        triggers: None,
    };
    
    create_symptom(State(app), Json(full_input)).await
}

pub async fn create_quick_medication(
    State(app): State<App>,
    Json(payload): Json<QuickMedicationInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/quick-medication");
    
    let full_input = MedicationInput {
        patient_id: payload.patient_id,
        name: payload.name,
        dosage: payload.dosage,
        frequency: payload.frequency,
        route: None,
        start_date: Utc::now(),
        end_date: None,
        prescribed_by: None,
        reason: None,
        side_effects: None,
    };
    
    create_medication(State(app), Json(full_input)).await
}
