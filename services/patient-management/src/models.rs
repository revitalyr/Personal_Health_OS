use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use validator::Validate;

mod types;
pub use types::*;

use crate::auth::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HospitalPatient {
    pub id: PatientId,
    pub profile_id: ProfileId,
    pub patient_id: ExternalPatientCode,

    pub blood_type: Option<BloodType>,

    pub emergency_contact_name: Option<ContactName>,
    pub emergency_contact_phone: Option<PhoneNumber>,

    pub insurance_provider: Option<InsuranceProvider>,
    pub insurance_policy_number: Option<PolicyNumber>,

    pub admission_date: Option<AdmissionDate>,
    pub discharge_date: Option<DischargeDate>,

    pub status: PatientStatus,

    pub created_at: CreatedAt,
    pub updated_at: UpdatedAt,
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
    pub id: EncounterId,
    pub patient_id: PatientId,
    pub doctor_id: Option<DoctorId>,

    pub encounter_type: EncounterType,

    pub start_time: EncounterStart,
    pub end_time: Option<EncounterEnd>,

    pub diagnosis: Option<Diagnosis>,
    pub treatment: Option<Treatment>,
    pub notes: Option<Notes>,

    pub created_at: CreatedAt,
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
    pub id: VitalsId,
    pub patient_id: PatientId,

    pub blood_pressure_systolic: Option<SystolicPressure>,
    pub blood_pressure_diastolic: Option<DiastolicPressure>,

    pub heart_rate: Option<HeartRate>,
    pub temperature: Option<BodyTemperature>,

    pub weight: Option<WeightKg>,
    pub height: Option<HeightCm>,

    pub oxygen_saturation: Option<OxygenSaturation>,

    pub recorded_at: RecordedAt,
    pub recorded_by: StaffId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAllergy {
    pub id: AllergyId,
    pub patient_id: PatientId,

    pub allergen: AllergenName,
    pub severity: AllergySeverity,

    pub reaction: Option<AllergyReaction>,
    pub notes: Option<AllergyNotes>,

    pub created_at: CreatedAt,
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
    pub id: MedicationRecordId,
    pub patient_id: PatientId,

    pub medication_name: MedicationName,
    pub dosage: Dosage,
    pub frequency: Frequency,
    pub route: AdministrationRoute,

    pub start_date: StartDate,
    pub end_date: Option<EndDate>,

    pub prescribed_by: PrescriberId,

    pub medication_status: MedicationStatus, 
    pub created_at: CreatedAt,
}

// Request/Response DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePatientRequest {
    pub profile_id: ProfileId,

    #[validate(length(min = 3, max = 50))]
    pub patient_id: ExternalPatientCode,

    pub blood_type: Option<BloodType>,

    pub emergency_contact_name: Option<ContactName>,

    #[validate(length(min = 10, max = 20))]
    pub emergency_contact_phone: Option<PhoneNumber>,

    pub insurance_provider: Option<InsuranceProvider>,
    pub insurance_policy_number: Option<PolicyNumber>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePatientRequest {
    pub blood_type: Option<BloodType>,

    pub emergency_contact_name: Option<ContactName>,
    pub emergency_contact_phone: Option<PhoneNumber>,

    pub insurance_provider: Option<InsuranceProvider>,
    pub insurance_policy_number: Option<PolicyNumber>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdmitPatientRequest {
    pub admission_date: AdmissionDate,
    pub notes: Option<Notes>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DischargePatientRequest {
    pub discharge_date: DischargeDate,
    pub discharge_notes: Option<Notes>,
    pub follow_up_instructions: Option<Notes>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEncounterRequest {
    pub doctor_id: Option<DoctorId>,
    pub encounter_type: EncounterType,
    pub start_time: EncounterStart,
    pub end_time: Option<EncounterEnd>,
    pub diagnosis: Option<Diagnosis>,
    pub treatment: Option<Treatment>,
    pub notes: Option<Notes>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RecordVitalsRequest {
    pub blood_pressure_systolic: Option<SystolicPressure>,
    pub blood_pressure_diastolic: Option<DiastolicPressure>,
    pub heart_rate: Option<HeartRate>,
    pub temperature: Option<BodyTemperature>,
    pub weight: Option<WeightKg>,
    pub height: Option<HeightCm>,
    pub oxygen_saturation: Option<OxygenSaturation>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddAllergyRequest {
    #[validate(length(min = 1, max = 255))]
    pub allergen: AllergenName,
    pub severity: AllergySeverity,
    pub reaction: Option<AllergyReaction>,
    pub notes: Option<AllergyNotes>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PrescribeMedicationRequest {
    #[validate(length(min = 1, max = 255))]
    pub medication_name: MedicationName,
    #[validate(length(min = 1, max = 100))]
    pub dosage: Dosage,
    #[validate(length(min = 1, max = 100))]
    pub frequency: Frequency,
    #[validate(length(min = 1, max = 50))]
    pub route: AdministrationRoute,
    pub start_date: StartDate,
    pub end_date: Option<EndDate>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct PatientResponse {
    pub id: PatientId,
    pub profile_id: ProfileId,
    pub patient_id: ExternalPatientCode,
    pub blood_type: Option<BloodType>,
    pub emergency_contact_name: Option<ContactName>,
    pub emergency_contact_phone: Option<PhoneNumber>,
    pub insurance_provider: Option<InsuranceProvider>,
    pub insurance_policy_number: Option<PolicyNumber>,
    pub admission_date: Option<AdmissionDate>,
    pub discharge_date: Option<DischargeDate>,
    pub status: PatientStatus,
    pub created_at: CreatedAt,
    pub updated_at: UpdatedAt,
    pub profile: Option<UserProfile>,
}

#[derive(Debug, Serialize)]
pub struct PatientSummaryResponse {
    pub id: PatientId,
    pub patient_id: ExternalPatientCode,
    pub name: String,
    pub age: Option<i32>,
    pub blood_type: Option<BloodType>,
    pub status: PatientStatus,
    pub admission_date: Option<AdmissionDate>,
    pub current_medications: i64,
    pub active_allergies: i64,
    pub recent_encounters: i64,
}

#[derive(Debug, Serialize)]
pub struct PatientTimelineResponse {
    pub patient_id: PatientId,
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub id: EncounterId,
    pub event_type: String,
    pub timestamp: RecordedAt,
    pub description: Notes,
    pub metadata: serde_json::Value,
}
