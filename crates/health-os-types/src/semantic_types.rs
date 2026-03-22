// Health OS Semantic Types Module
// This module provides semantic type aliases that improve code readability and expressiveness
// while maintaining type safety and self-documenting code properties.

use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// CORE SEMANTIC TYPES
// ============================================================================

/// Unique identifier for a patient in the system
pub type PatientId = Uuid;

/// Unique identifier for a patient's profile
pub type ProfileId = Uuid;

/// External patient code from legacy systems
pub type ExternalPatientCode = String;

/// Unique identifier for medical encounters
pub type EncounterId = Uuid;

/// Unique identifier for doctors/staff
pub type DoctorId = Uuid;

/// Unique identifier for staff members
pub type StaffId = Uuid;

/// Unique identifier for vitals records
pub type VitalsId = Uuid;

/// Unique identifier for allergy records
pub type AllergyId = Uuid;

/// Unique identifier for medication records
pub type MedicationRecordId = Uuid;

/// Unique identifier for prescribers
pub type PrescriberId = Uuid;

// ============================================================================
// MEDICAL DATA TYPES
// ============================================================================

/// Blood type with semantic meaning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Patient contact name with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactName(String);

impl ContactName {
    pub fn new(name: &str) -> Result<Self, &'static str> {
        if name.trim().is_empty() {
            Err("Contact name cannot be empty")
        } else if name.len() > 100 {
            Err("Contact name too long")
        } else {
            Ok(ContactName(name.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Phone number with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(phone: &str) -> Result<Self, &'static str> {
        let cleaned = phone.chars().filter(|c| c.is_numeric()).collect::<String>();
        if cleaned.len() < 10 || cleaned.len() > 15 {
            Err("Invalid phone number format")
        } else {
            Ok(PhoneNumber(cleaned))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Insurance provider name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProvider(String);

impl InsuranceProvider {
    pub fn new(provider: &str) -> Result<Self, &'static str> {
        if provider.trim().is_empty() {
            Err("Insurance provider cannot be empty")
        } else if provider.len() > 200 {
            Err("Insurance provider name too long")
        } else {
            Ok(InsuranceProvider(provider.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Insurance policy number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyNumber(String);

impl PolicyNumber {
    pub fn new(policy: &str) -> Result<Self, &'static str> {
        if policy.trim().is_empty() {
            Err("Policy number cannot be empty")
        } else if policy.len() > 50 {
            Err("Policy number too long")
        } else {
            Ok(PolicyNumber(policy.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Allergen name with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergenName(String);

impl AllergenName {
    pub fn new(allergen: &str) -> Result<Self, &'static str> {
        if allergen.trim().is_empty() {
            Err("Allergen name cannot be empty")
        } else if allergen.len() > 100 {
            Err("Allergen name too long")
        } else {
            Ok(AllergenName(allergen.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Allergy reaction description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergyReaction(String);

impl AllergyReaction {
    pub fn new(reaction: &str) -> Result<Self, &'static str> {
        if reaction.len() > 500 {
            Err("Allergy reaction description too long")
        } else {
            Ok(AllergyReaction(reaction.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Allergy notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergyNotes(String);

impl AllergyNotes {
    pub fn new(notes: &str) -> Result<Self, &'static str> {
        if notes.len() > 1000 {
            Err("Allergy notes too long")
        } else {
            Ok(AllergyNotes(notes.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Medication name with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationName(String);

impl MedicationName {
    pub fn new(name: &str) -> Result<Self, &'static str> {
        if name.trim().is_empty() {
            Err("Medication name cannot be empty")
        } else if name.len() > 200 {
            Err("Medication name too long")
        } else {
            Ok(MedicationName(name.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Dosage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dosage(String);

impl Dosage {
    pub fn new(dosage: &str) -> Result<Self, &'static str> {
        if dosage.trim().is_empty() {
            Err("Dosage cannot be empty")
        } else if dosage.len() > 100 {
            Err("Dosage description too long")
        } else {
            Ok(Dosage(dosage.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Medication frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequency(String);

impl Frequency {
    pub fn new(frequency: &str) -> Result<Self, &'static str> {
        if frequency.trim().is_empty() {
            Err("Frequency cannot be empty")
        } else if frequency.len() > 100 {
            Err("Frequency description too long")
        } else {
            Ok(Frequency(frequency.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// MEDICAL MEASUREMENT TYPES
// ============================================================================

/// Systolic blood pressure value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystolicPressure(i32);

impl SystolicPressure {
    pub fn new(value: i32) -> Result<Self, &'static str> {
        if value < 60 || value > 250 {
            Err("Systolic pressure out of normal range")
        } else {
            Ok(SystolicPressure(value))
        }
    }
    
    pub fn value(&self) -> i32 {
        self.0
    }
}

/// Diastolic blood pressure value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiastolicPressure(i32);

impl DiastolicPressure {
    pub fn new(value: i32) -> Result<Self, &'static str> {
        if value < 40 || value > 150 {
            Err("Diastolic pressure out of normal range")
        } else {
            Ok(DiastolicPressure(value))
        }
    }
    
    pub fn value(&self) -> i32 {
        self.0
    }
}

/// Heart rate measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRate(i32);

impl HeartRate {
    pub fn new(value: i32) -> Result<Self, &'static str> {
        if value < 30 || value > 220 {
            Err("Heart rate out of normal range")
        } else {
            Ok(HeartRate(value))
        }
    }
    
    pub fn value(&self) -> i32 {
        self.0
    }
}

/// Body temperature in Celsius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyTemperature(f32);

impl BodyTemperature {
    pub fn new(value: f32) -> Result<Self, &'static str> {
        if value < 30.0 || value > 45.0 {
            Err("Body temperature out of normal range")
        } else {
            Ok(BodyTemperature(value))
        }
    }
    
    pub fn value(&self) -> f32 {
        self.0
    }
}

/// Weight in kilograms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightKg(f32);

impl WeightKg {
    pub fn new(value: f32) -> Result<Self, &'static str> {
        if value < 1.0 || value > 500.0 {
            Err("Weight out of normal range")
        } else {
            Ok(WeightKg(value))
        }
    }
    
    pub fn value(&self) -> f32 {
        self.0
    }
}

/// Height in centimeters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightCm(f32);

impl HeightCm {
    pub fn new(value: f32) -> Result<Self, &'static str> {
        if value < 30.0 || value > 250.0 {
            Err("Height out of normal range")
        } else {
            Ok(HeightCm(value))
        }
    }
    
    pub fn value(&self) -> f32 {
        self.0
    }
}

/// Oxygen saturation percentage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxygenSaturation(f32);

impl OxygenSaturation {
    pub fn new(value: f32) -> Result<Self, &'static str> {
        if value < 0.0 || value > 100.0 {
            Err("Oxygen saturation out of valid range")
        } else {
            Ok(OxygenSaturation(value))
        }
    }
    
    pub fn value(&self) -> f32 {
        self.0
    }
}

// ============================================================================
// TEMPORAL TYPES
// ============================================================================

/// Creation timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedAt(pub DateTime<Utc>);

impl CreatedAt {
    pub fn now() -> Self {
        CreatedAt(Utc::now())
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Update timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatedAt(pub DateTime<Utc>);

impl UpdatedAt {
    pub fn now() -> Self {
        UpdatedAt(Utc::now())
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Admission date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionDate(pub DateTime<Utc>);

impl AdmissionDate {
    pub fn new(date: DateTime<Utc>) -> Self {
        AdmissionDate(date)
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Discharge date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DischargeDate(pub DateTime<Utc>);

impl DischargeDate {
    pub fn new(date: DateTime<Utc>) -> Self {
        DischargeDate(date)
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Encounter start time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterStart(pub DateTime<Utc>);

impl EncounterStart {
    pub fn new(time: DateTime<Utc>) -> Self {
        EncounterStart(time)
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Encounter end time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterEnd(pub DateTime<Utc>);

impl EncounterEnd {
    pub fn new(time: DateTime<Utc>) -> Self {
        EncounterEnd(time)
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Vitals recording timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedAt(pub DateTime<Utc>);

impl RecordedAt {
    pub fn now() -> Self {
        RecordedAt(Utc::now())
    }
    
    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Medication start date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartDate(pub NaiveDate);

impl StartDate {
    pub fn new(date: NaiveDate) -> Self {
        StartDate(date)
    }
    
    pub fn value(&self) -> NaiveDate {
        self.0
    }
}

/// Medication end date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndDate(pub NaiveDate);

impl EndDate {
    pub fn new(date: NaiveDate) -> Self {
        EndDate(date)
    }
    
    pub fn value(&self) -> NaiveDate {
        self.0
    }
}

// ============================================================================
// MEDICAL CONTENT TYPES
// ============================================================================

/// Medical diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis(String);

impl Diagnosis {
    pub fn new(diagnosis: &str) -> Result<Self, &'static str> {
        if diagnosis.trim().is_empty() {
            Err("Diagnosis cannot be empty")
        } else if diagnosis.len() > 1000 {
            Err("Diagnosis too long")
        } else {
            Ok(Diagnosis(diagnosis.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Medical treatment description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treatment(String);

impl Treatment {
    pub fn new(treatment: &str) -> Result<Self, &'static str> {
        if treatment.len() > 2000 {
            Err("Treatment description too long")
        } else {
            Ok(Treatment(treatment.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Medical notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notes(String);

impl Notes {
    pub fn new(notes: &str) -> Result<Self, &'static str> {
        if notes.len() > 5000 {
            Err("Notes too long")
        } else {
            Ok(Notes(notes.to_string()))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// ENUMS WITH SEMANTIC MEANING
// ============================================================================

/// Patient admission status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatientStatus {
    Admitted,
    Discharged,
    Outpatient,
    Emergency,
    Transferred,
}

/// Medical encounter type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncounterType {
    Consultation,
    Emergency,
    Surgery,
    FollowUp,
    Diagnostic,
    Therapy,
}

/// Allergy severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllergySeverity {
    Mild,
    Moderate,
    Severe,
    LifeThreatening,
}

/// Medication administration routes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdministrationRoute {
    Oral,
    Intravenous,
    Intramuscular,
    Subcutaneous,
    Topical,
    Inhalation,
}

/// Medication status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MedicationStatus {
    Active,
    Completed,
    Discontinued,
    OnHold,
}

// ============================================================================
// VALIDATION TRAITS
// ============================================================================

/// Trait for validating semantic types
pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
}

/// Trait for converting to string representation
pub trait AsString {
    fn as_string(&self) -> String;
}

// Implement validation traits for semantic types
impl Validatable for ContactName {
    fn validate(&self) -> Result<(), String> {
        if self.0.is_empty() {
            Err("Contact name cannot be empty".to_string())
        } else {
            Ok(())
        }
    }
}

impl Validatable for PhoneNumber {
    fn validate(&self) -> Result<(), String> {
        let digits: String = self.0.chars().filter(|c| c.is_numeric()).collect();
        if digits.len() < 10 || digits.len() > 15 {
            Err("Invalid phone number format".to_string())
        } else {
            Ok(())
        }
    }
}

// Implement string conversion traits
impl AsString for ContactName {
    fn as_string(&self) -> String {
        self.0.clone()
    }
}

impl AsString for PhoneNumber {
    fn as_string(&self) -> String {
        self.0.clone()
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Create a new patient ID
pub fn new_patient_id() -> PatientId {
    Uuid::new_v4()
}

/// Create a new encounter ID
pub fn new_encounter_id() -> EncounterId {
    Uuid::new_v4()
}

/// Create a new staff ID
pub fn new_staff_id() -> StaffId {
    Uuid::new_v4()
}

/// Validate blood pressure reading
pub fn validate_blood_pressure(systolic: i32, diastolic: i32) -> Result<(SystolicPressure, DiastolicPressure), String> {
    let systolic_pressure = SystolicPressure::new(systolic)
        .map_err(|e| format!("Invalid systolic pressure: {}", e))?;
    
    let diastolic_pressure = DiastolicPressure::new(diastolic)
        .map_err(|e| format!("Invalid diastolic pressure: {}", e))?;
    
    // Ensure systolic is higher than diastolic
    if systolic <= diastolic {
        return Err("Systolic pressure must be higher than diastolic pressure".to_string());
    }
    
    Ok((systolic_pressure, diastolic_pressure))
}

/// Format phone number for display
pub fn format_phone_number(phone: &PhoneNumber) -> String {
    let digits = &phone.0;
    match digits.len() {
        10 => format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10]),
        11 if digits.starts_with('1') => format!("+1 ({}) {}-{}", &digits[1..4], &digits[4..7], &digits[7..11]),
        _ => digits.to_string(),
    }
}

// ============================================================================
// MODULE EXPORTS
// ============================================================================

// Note: Types are used directly via crate::semantic_types::*
