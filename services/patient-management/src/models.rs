use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HospitalPatient {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub patient_id: String,
    pub blood_type: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub insurance_provider: Option<String>,
    pub insurance_policy_number: Option<String>,
    pub admission_date: Option<DateTime<Utc>>,
    pub discharge_date: Option<DateTime<Utc>>,
    pub status: PatientStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum PatientStatus {
    Active,
    Discharged,
    Transferred,
    Deceased,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MedicalEncounter {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Option<Uuid>,
    pub encounter_type: EncounterType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum EncounterType {
    Admission,
    Consultation,
    Procedure,
    Surgery,
    Emergency,
    FollowUp,
    Discharge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientVitals {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub blood_pressure_systolic: Option<i32>,
    pub blood_pressure_diastolic: Option<i32>,
    pub heart_rate: Option<i32>,
    pub temperature: Option<f32>,
    pub weight: Option<f32>,
    pub height: Option<f32>,
    pub oxygen_saturation: Option<f32>,
    pub recorded_at: DateTime<Utc>,
    pub recorded_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAllergy {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub allergen: String,
    pub severity: AllergySeverity,
    pub reaction: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum AllergySeverity {
    Mild,
    Moderate,
    Severe,
    LifeThreatening,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientMedication {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub medication_name: String,
    pub dosage: String,
    pub frequency: String,
    pub route: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub prescribed_by: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePatientRequest {
    #[validate(length(min = 1, max = 255))]
    pub profile_id: Uuid,
    #[validate(length(min = 3, max = 50))]
    pub patient_id: String,
    pub blood_type: Option<String>,
    pub emergency_contact_name: Option<String>,
    #[validate(length(min = 10, max = 20))]
    pub emergency_contact_phone: Option<String>,
    pub insurance_provider: Option<String>,
    pub insurance_policy_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePatientRequest {
    pub blood_type: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub insurance_provider: Option<String>,
    pub insurance_policy_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdmitPatientRequest {
    pub admission_date: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DischargePatientRequest {
    pub discharge_date: DateTime<Utc>,
    pub discharge_notes: Option<String>,
    pub follow_up_instructions: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEncounterRequest {
    pub doctor_id: Option<Uuid>,
    pub encounter_type: EncounterType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RecordVitalsRequest {
    pub blood_pressure_systolic: Option<i32>,
    pub blood_pressure_diastolic: Option<i32>,
    pub heart_rate: Option<i32>,
    pub temperature: Option<f32>,
    pub weight: Option<f32>,
    pub height: Option<f32>,
    pub oxygen_saturation: Option<f32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddAllergyRequest {
    #[validate(length(min = 1, max = 255))]
    pub allergen: String,
    pub severity: AllergySeverity,
    pub reaction: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PrescribeMedicationRequest {
    #[validate(length(min = 1, max = 255))]
    pub medication_name: String,
    #[validate(length(min = 1, max = 100))]
    pub dosage: String,
    #[validate(length(min = 1, max = 100))]
    pub frequency: String,
    #[validate(length(min = 1, max = 50))]
    pub route: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct PatientResponse {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub patient_id: String,
    pub blood_type: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub insurance_provider: Option<String>,
    pub insurance_policy_number: Option<String>,
    pub admission_date: Option<DateTime<Utc>>,
    pub discharge_date: Option<DateTime<Utc>>,
    pub status: PatientStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub profile: Option<crate::auth::UserProfile>,
}

#[derive(Debug, Serialize)]
pub struct PatientSummaryResponse {
    pub id: Uuid,
    pub patient_id: String,
    pub name: String,
    pub age: Option<i32>,
    pub blood_type: Option<String>,
    pub status: PatientStatus,
    pub admission_date: Option<DateTime<Utc>>,
    pub current_medications: i64,
    pub active_allergies: i64,
    pub recent_encounters: i64,
}

#[derive(Debug, Serialize)]
pub struct PatientTimelineResponse {
    pub patient_id: Uuid,
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub metadata: serde_json::Value,
}
