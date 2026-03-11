use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

use crate::{app::App, trace_request, trace_response};

// Input structures
#[derive(Debug, Deserialize)]
pub struct SymptomInput {
    pub patient_id: Uuid,
    pub name: String,
    pub severity: u8,
    pub description: Option<String>,
    pub duration: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MedicationInput {
    pub patient_id: Uuid,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub prescribed_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LabResultInput {
    pub patient_id: Uuid,
    pub test_name: String,
    pub value: String,
    pub unit: String,
    pub reference_range: Option<String>,
    pub status: String,
    pub facility: String,
    pub test_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct DoctorVisitInput {
    pub patient_id: Uuid,
    pub doctor_name: String,
    pub specialty: String,
    pub facility: String,
    pub visit_date: DateTime<Utc>,
    pub reason: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DiagnosisInput {
    pub patient_id: Uuid,
    pub condition: String,
    pub icd10_code: Option<String>,
    pub diagnosed_by: String,
    pub diagnosis_date: NaiveDate,
    pub acute: bool,
}

#[derive(Debug, Deserialize)]
pub struct ManualEntryInput {
    pub patient_id: Uuid,
    pub entry_type: String,
    pub title: String,
    pub description: String,
    pub entry_date: DateTime<Utc>,
    pub category: Option<String>,
}

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

// Symptom handlers
pub async fn create_symptom(
    State(_app): State<App>,
    Json(payload): Json<SymptomInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/symptom");
    
    // Validate severity
    if payload.severity < 1 || payload.severity > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // TODO: Call document processor service
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "symptom",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Symptom creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

pub async fn create_quick_symptom(
    State(_app): State<App>,
    Json(payload): Json<QuickSymptomInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/quick-symptom");
    
    // Validate severity
    if payload.severity < 1 || payload.severity > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "symptom",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Quick symptom creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(10));
    Ok(Json(response))
}

// Medication handlers
pub async fn create_medication(
    State(_app): State<App>,
    Json(payload): Json<MedicationInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/medication");
    
    // Validate dates
    if let Some(end_date) = payload.end_date {
        if end_date <= payload.start_date {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "medication",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Medication creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

pub async fn create_quick_medication(
    State(_app): State<App>,
    Json(payload): Json<QuickMedicationInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/quick-medication");
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "medication",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Quick medication creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(10));
    Ok(Json(response))
}

// Lab result handler
pub async fn create_lab_result(
    State(_app): State<App>,
    Json(payload): Json<LabResultInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/lab-result");
    
    // Validate status
    let valid_statuses = ["normal", "high", "low", "critical", "borderline"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "lab_result",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Lab result creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

// Doctor visit handler
pub async fn create_doctor_visit(
    State(_app): State<App>,
    Json(payload): Json<DoctorVisitInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/doctor-visit");
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "doctor_visit",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Doctor visit creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(20));
    Ok(Json(response))
}

// Diagnosis handler
pub async fn create_diagnosis(
    State(_app): State<App>,
    Json(payload): Json<DiagnosisInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/diagnosis");
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "diagnosis",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Diagnosis creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}

// Manual entry handler
pub async fn create_manual_entry(
    State(_app): State<App>,
    Json(payload): Json<ManualEntryInput>,
) -> Result<Json<Value>, StatusCode> {
    trace_request!("POST", "/input/manual");
    
    let event_id = Uuid::new_v4();
    
    let response = serde_json::json!({
        "event_id": event_id,
        "type": "manual_entry",
        "status": "created",
        "patient_id": payload.patient_id,
        "message": "Manual entry creation forwarded to processor"
    });

    trace_response!(StatusCode::CREATED, std::time::Duration::from_millis(15));
    Ok(Json(response))
}
