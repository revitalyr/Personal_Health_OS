use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::{PatientService, PatientError, CreatePatientRequest, UpdatePatientRequest, AdmitPatientRequest, DischargePatientRequest, CreateEncounterRequest, RecordVitalsRequest, AddAllergyRequest, PrescribeMedicationRequest, PatientTimelineResponse};

#[derive(Debug, Deserialize)]
pub struct ListPatientsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListEncountersQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListVitalsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GetTimelineQuery {
    pub limit: Option<i64>,
}

pub fn patient_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/patients", post(create_patient))
        .route("/patients", get(list_patients))
        .route("/patients/:id", get(get_patient))
        .route("/patients/:id", put(update_patient))
        .route("/patients/:id/admit", post(admit_patient))
        .route("/patients/:id/discharge", post(discharge_patient))
        .route("/patients/:id/encounters", post(create_encounter))
        .route("/patients/:id/encounters", get(get_patient_encounters))
        .route("/patients/:id/vitals", post(record_vitals))
        .route("/patients/:id/vitals", get(get_patient_vitals))
        .route("/patients/:id/allergies", post(add_allergy))
        .route("/patients/:id/allergies", get(get_patient_allergies))
        .route("/patients/:id/medications", post(prescribe_medication))
        .route("/patients/:id/medications", get(get_patient_medications))
        .route("/patients/:id/timeline", get(get_patient_timeline))
        .route("/patients/search/:patient_id", get(search_by_patient_id))
}

async fn create_patient(
    State(state): State<crate::AppState>,
    Json(request): Json<CreatePatientRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    
    // Validate request
    if let Err(validation_error) = validator::Validate::validate(&request) {
        return Err(PatientError::Validation(format!("{:?}", validation_error)));
    }
    
    let patient = patient_service.create_patient(request).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patient
    })))
}

async fn list_patients(
    State(state): State<crate::AppState>,
    Query(query): Query<ListPatientsQuery>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let status = query.status.and_then(|s| match s.as_str() {
        "active" => Some(crate::models::PatientStatus::Active),
        "discharged" => Some(crate::models::PatientStatus::Discharged),
        "transferred" => Some(crate::models::PatientStatus::Transferred),
        "deceased" => Some(crate::models::PatientStatus::Deceased),
        _ => None,
    });
    
    let patients = patient_service.list_patients(limit, offset, status).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patients,
        "pagination": {
            "limit": limit,
            "offset": offset,
            "total": patients.len()
        }
    })))
}

async fn get_patient(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let patient = patient_service.get_patient(patient_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patient
    })))
}

async fn update_patient(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<UpdatePatientRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let patient = patient_service.update_patient(patient_id, request).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patient
    })))
}

async fn admit_patient(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<AdmitPatientRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let patient = patient_service.admit_patient(patient_id, request).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patient,
        "message": "Patient admitted successfully"
    })))
}

async fn discharge_patient(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<DischargePatientRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let patient = patient_service.discharge_patient(patient_id, request).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patient,
        "message": "Patient discharged successfully"
    })))
}

async fn create_encounter(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<CreateEncounterRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let encounter = patient_service.create_encounter(request, patient_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": encounter,
        "message": "Medical encounter created successfully"
    })))
}

async fn get_patient_encounters(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<ListEncountersQuery>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let limit = query.limit.unwrap_or(20);
    let encounters = patient_service.get_patient_encounters(patient_id, limit).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": encounters
    })))
}

async fn record_vitals(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<RecordVitalsRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    
    // For now, we'll use a fixed recorded_by ID. In a real implementation,
    // this would come from the authenticated user
    let recorded_by = Uuid::new_v4();
    
    let vitals = patient_service.record_vitals(patient_id, request, recorded_by).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": vitals,
        "message": "Vitals recorded successfully"
    })))
}

async fn get_patient_vitals(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<ListVitalsQuery>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let limit = query.limit.unwrap_or(50);
    let vitals = patient_service.get_patient_vitals(patient_id, limit).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": vitals
    })))
}

async fn add_allergy(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<AddAllergyRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let allergy = patient_service.add_allergy(patient_id, request).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": allergy,
        "message": "Allergy added successfully"
    })))
}

async fn get_patient_allergies(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let allergies = patient_service.get_patient_allergies(patient_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": allergies
    })))
}

async fn prescribe_medication(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Json(request): Json<PrescribeMedicationRequest>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    
    // For now, we'll use a fixed prescribed_by ID. In a real implementation,
    // this would come from the authenticated user
    let prescribed_by = Uuid::new_v4();
    
    let medication = patient_service.prescribe_medication(patient_id, request, prescribed_by).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": medication,
        "message": "Medication prescribed successfully"
    })))
}

async fn get_patient_medications(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let active_only = query.get("active_only").map(|v| v == "true").unwrap_or(true);
    let medications = patient_service.get_patient_medications(patient_id, active_only).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": medications
    })))
}

async fn get_patient_timeline(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<GetTimelineQuery>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let limit = query.limit.unwrap_or(100);
    let timeline = patient_service.get_patient_timeline(patient_id, limit).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": timeline
    })))
}

async fn search_by_patient_id(
    State(state): State<crate::AppState>,
    Path(patient_id): Path<String>,
) -> Result<Json<serde_json::Value>, PatientError> {
    let patient_service = PatientService::new(state.db.clone());
    let patient = patient_service.get_patient_by_patient_id(&patient_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": patient
    })))
}
