// Health OS Semantic Types Module - Fixed Version
// This module provides semantic type aliases that improve code readability and expressiveness
// while maintaining type safety and self-documenting code properties.

pub mod semantic_types;
pub mod hospital_patient;
pub mod medical_encounter;
pub mod patient_vitals;
pub mod patient_allergy;
pub mod patient_medication;

// Re-export all modules for convenient use
pub use {
    hospital_patient::HospitalPatient,
    medical_encounter::MedicalEncounter,
    patient_vitals::{PatientVitals, VitalsSummary},
    patient_allergy::{PatientAllergy, AllergyCategory, AllergySummary},
    patient_medication::{PatientMedication, MedicationSummary},
    semantic_types::*,
};

// Simplified validation request types using semantic types
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Create patient request with semantic type validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePatientRequest {
    /// Reference to patient's profile information
    #[validate(length(min = 1))]
    pub profile_id: ProfileId,
    
    /// External patient code from legacy systems
    #[validate(length(min = 3, max = 50))]
    pub patient_id: String,
    
    /// Patient's blood type
    pub blood_type: Option<BloodType>,
    
    /// Emergency contact name
    pub emergency_contact_name: Option<String>,
    
    /// Emergency contact phone
    #[validate(length(min = 10, max = 20))]
    pub emergency_contact_phone: Option<String>,
    
    /// Insurance provider name
    pub insurance_provider: Option<String>,
    
    /// Insurance policy number
    pub insurance_policy_number: Option<String>,
}

impl CreatePatientRequest {
    /// Validate and create a new patient from this request
    pub fn validate_request(&self) -> Result<(), String> {
        // Basic validation - semantic types handle detailed validation
        if let Some(ref phone) = self.emergency_contact_phone {
            if phone.len() < 10 || phone.len() > 20 {
                return Err("Invalid emergency contact phone".to_string());
            }
        }
        
        if let Some(ref contact_name) = self.emergency_contact_name {
            if contact_name.trim().is_empty() {
                return Err("Emergency contact name cannot be empty".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Convert to HospitalPatient
    pub fn to_hospital_patient(&self) -> Result<HospitalPatient, String> {
        self.validate_request()?;
        
        let mut patient = HospitalPatient::new(
            self.profile_id,
            self.patient_id.clone(),
        );
        
        // Set optional fields
        if let Some(ref blood_type) = self.blood_type {
            patient.blood_type = Some(blood_type.clone());
        }
        
        if let Some(ref contact_name) = self.emergency_contact_name {
            if let Ok(validated_name) = ContactName::new(contact_name) {
                patient.emergency_contact_name = Some(validated_name);
            }
        }
        
        if let Some(ref phone) = self.emergency_contact_phone {
            if let Ok(validated_phone) = PhoneNumber::new(phone) {
                patient.emergency_contact_phone = Some(validated_phone);
            }
        }
        
        if let Some(ref provider) = self.insurance_provider {
            if let Ok(validated_provider) = InsuranceProvider::new(provider) {
                patient.insurance_provider = Some(validated_provider);
            }
        }
        
        if let Some(ref policy) = self.insurance_policy_number {
            if let Ok(validated_policy) = PolicyNumber::new(policy) {
                patient.insurance_policy_number = Some(validated_policy);
            }
        }
        
        Ok(patient)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_create_patient_request_validation() {
        let profile_id = ProfileId::new_v4();
        
        // Valid request
        let valid_request = CreatePatientRequest {
            profile_id,
            patient_id: "P12345".to_string(),
            blood_type: Some(BloodType::OPositive),
            emergency_contact_name: Some("John Doe".to_string()),
            emergency_contact_phone: Some("5551234567".to_string()),
            insurance_provider: Some("Blue Cross".to_string()),
            insurance_policy_number: Some("BC123456789".to_string()),
        };
        
        assert!(valid_request.validate_request().is_ok());
        
        let patient = valid_request.to_hospital_patient();
        assert!(patient.is_ok());
    }
    
    #[test]
    fn test_create_patient_request_invalid_phone() {
        let profile_id = ProfileId::new_v4();
        
        let invalid_request = CreatePatientRequest {
            profile_id,
            patient_id: "P12345".to_string(),
            blood_type: Some(BloodType::OPositive),
            emergency_contact_name: Some("John Doe".to_string()),
            emergency_contact_phone: Some("123".to_string()), // Invalid phone
            insurance_provider: Some("Blue Cross".to_string()),
            insurance_policy_number: Some("BC123456789".to_string()),
        };
        
        assert!(invalid_request.validate_request().is_err());
        assert!(invalid_request.to_hospital_patient().is_err());
    }
    
    #[test]
    fn test_semantic_type_benefits() {
        // This test demonstrates the benefits of semantic types
        
        // Before semantic types:
        // pub emergency_contact_phone: Option<String>,
        // pub blood_pressure_systolic: Option<i32>,
        
        // After semantic types:
        // pub emergency_contact_phone: Option<PhoneNumber>,
        // pub blood_pressure_systolic: Option<SystolicPressure>,
        
        // Benefits:
        // 1. Type safety - invalid values are caught at creation time
        // 2. Self-documenting - PhoneNumber clearly indicates expected format
        // 3. Validation - built-in validation logic
        // 4. Methods - semantic methods like format_phone_number()
        
        let valid_phone = PhoneNumber::new("5551234567").unwrap();
        let formatted = format_phone_number(&valid_phone);
        assert!(formatted.contains("(") && formatted.contains(")"));
        
        let valid_pressure = SystolicPressure::new(120).unwrap();
        assert_eq!(valid_pressure.value(), 120);
        
        let invalid_pressure = SystolicPressure::new(300);
        assert!(invalid_pressure.is_err());
    }
}
