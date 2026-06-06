//! Semantic type definitions for the patient management service
//! 
//! This module provides strongly-typed wrappers around common data types
//! to improve type safety and code readability.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

// ==================== ID TYPES ====================

/// Strongly typed patient identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct PatientId(pub Uuid);

/// Strongly typed profile identifier  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProfileId(pub Uuid);

/// Strongly typed doctor identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct DoctorId(pub Uuid);

/// Strongly typed staff identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct StaffId(pub Uuid);

/// Strongly typed encounter identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncounterId(pub Uuid);

/// Strongly typed vitals identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct VitalsId(pub Uuid);

/// Strongly typed allergy identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct AllergyId(pub Uuid);

/// Strongly typed medication record identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct MedicationRecordId(pub Uuid);

/// Strongly typed prescriber identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrescriberId(pub Uuid);

// ==================== EXTERNAL CODE TYPES ====================

/// External patient code from other systems
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalPatientCode(pub String);

/// Contact name with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContactName(pub String);

/// Phone number with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhoneNumber(pub String);

/// Insurance provider name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InsuranceProvider(pub String);

/// Insurance policy number
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyNumber(pub String);

// ==================== MEDICAL TYPES ====================

/// Blood type with proper enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BloodType {
    APositive,
    ANegative,
    BPositive,
    BNegative,
    ABPositive,
    ABNegative,
    OPositive,
    ONegative,
}

/// Allergen name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AllergenName(pub String);

/// Allergy reaction description
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AllergyReaction(pub String);

/// Allergy notes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AllergyNotes(pub String);

/// Medication name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MedicationName(pub String);

/// Dosage information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dosage(pub String);

/// Frequency information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Frequency(pub String);

/// Administration route
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum AdministrationRoute {
    Oral,
    Intravenous,
    Intramuscular,
    Subcutaneous,
    Topical,
    Inhalation,
}

/// Diagnosis description
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Diagnosis(pub String);

/// Treatment description
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Treatment(pub String);

/// Medical notes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Notes(pub String);

// ==================== VITAL SIGNS ====================

/// Systolic blood pressure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystolicPressure(pub i32);

/// Diastolic blood pressure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiastolicPressure(pub i32);

/// Heart rate in beats per minute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeartRate(pub i32);

/// Body temperature in Celsius
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodyTemperature(pub f32);

/// Weight in kilograms
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeightKg(pub f32);

/// Height in centimeters
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeightCm(pub f32);

/// Oxygen saturation percentage
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OxygenSaturation(pub f32);

// ==================== DATE/TIME TYPES ====================

/// Admission date
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdmissionDate(pub DateTime<Utc>);

/// Discharge date
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DischargeDate(pub DateTime<Utc>);

/// Start date for medication or treatment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StartDate(pub NaiveDate);

/// End date for medication or treatment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EndDate(pub NaiveDate);

/// Encounter start time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncounterStart(pub DateTime<Utc>);

/// Encounter end time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncounterEnd(pub DateTime<Utc>);

/// Recorded at timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordedAt(pub DateTime<Utc>);

/// Created at timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CreatedAt(pub DateTime<Utc>);

/// Updated at timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UpdatedAt(pub DateTime<Utc>);

// ==================== STATUS TYPES ====================

/// Medication status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum MedicationStatus {
    Active,
    Completed,
    Discontinued,
}

/// Patient status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum PatientStatus {
    Active,
    Discharged,
    Transferred,
    Deceased,
}

/// Allergy severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum AllergySeverity {
    Mild,
    Moderate,
    Severe,
    LifeThreatening,
}

/// Encounter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
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

// ==================== IMPLEMENTATIONS ====================

// Implement From traits for conversions
impl From<Uuid> for PatientId {
    fn from(id: Uuid) -> Self {
        PatientId(id)
    }
}

impl From<PatientId> for Uuid {
    fn from(id: PatientId) -> Self {
        id.0
    }
}

impl From<Uuid> for ProfileId {
    fn from(id: Uuid) -> Self {
        ProfileId(id)
    }
}

impl From<ProfileId> for Uuid {
    fn from(id: ProfileId) -> Self {
        id.0
    }
}

impl From<String> for ExternalPatientCode {
    fn from(code: String) -> Self {
        ExternalPatientCode(code)
    }
}

impl From<&str> for ExternalPatientCode {
    fn from(code: &str) -> Self {
        ExternalPatientCode(code.to_string())
    }
}

impl From<String> for ContactName {
    fn from(name: String) -> Self {
        ContactName(name)
    }
}

impl From<&str> for ContactName {
    fn from(name: &str) -> Self {
        ContactName(name.to_string())
    }
}

impl From<String> for PhoneNumber {
    fn from(phone: String) -> Self {
        PhoneNumber(phone)
    }
}

impl From<&str> for PhoneNumber {
    fn from(phone: &str) -> Self {
        PhoneNumber(phone.to_string())
    }
}

impl From<DateTime<Utc>> for AdmissionDate {
    fn from(date: DateTime<Utc>) -> Self {
        AdmissionDate(date)
    }
}

impl From<AdmissionDate> for DateTime<Utc> {
    fn from(date: AdmissionDate) -> Self {
        date.0
    }
}

impl From<NaiveDate> for StartDate {
    fn from(date: NaiveDate) -> Self {
        StartDate(date)
    }
}

impl From<StartDate> for NaiveDate {
    fn from(date: StartDate) -> Self {
        date.0
    }
}

// Implement Display for better debugging
impl std::fmt::Display for PatientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for ExternalPatientCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for BloodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BloodType::APositive => write!(f, "A+"),
            BloodType::ANegative => write!(f, "A-"),
            BloodType::BPositive => write!(f, "B+"),
            BloodType::BNegative => write!(f, "B-"),
            BloodType::ABPositive => write!(f, "AB+"),
            BloodType::ABNegative => write!(f, "AB-"),
            BloodType::OPositive => write!(f, "O+"),
            BloodType::ONegative => write!(f, "O-"),
        }
    }
}

// Implement validation methods
impl PhoneNumber {
    /// Validate phone number format (basic validation)
    pub fn is_valid(&self) -> bool {
        let phone = &self.0;
        phone.len() >= 10 && phone.chars().filter(|c| c.is_ascii_digit()).count() >= 10
    }
}

impl SystolicPressure {
    /// Check if systolic pressure is within normal range
    pub fn is_normal(&self) -> bool {
        self.0 >= 90 && self.0 <= 120
    }
}

impl DiastolicPressure {
    /// Check if diastolic pressure is within normal range
    pub fn is_normal(&self) -> bool {
        self.0 >= 60 && self.0 <= 80
    }
}

impl HeartRate {
    /// Check if heart rate is within normal range for adults
    pub fn is_normal(&self) -> bool {
        self.0 >= 60 && self.0 <= 100
    }
}

impl BodyTemperature {
    /// Check if temperature is within normal range (Celsius)
    pub fn is_normal(&self) -> bool {
        self.0 >= 36.1 && self.0 <= 37.2
    }
}

impl OxygenSaturation {
    /// Check if oxygen saturation is within normal range
    pub fn is_normal(&self) -> bool {
        self.0 >= 95.0 && self.0 <= 100.0
    }
}
