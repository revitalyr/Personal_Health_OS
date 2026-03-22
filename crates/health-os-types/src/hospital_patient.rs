// Refactored Hospital Patient using semantic types
// This demonstrates how semantic types improve readability and type safety

use crate::semantic_types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Hospital patient with semantic type aliases for improved readability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HospitalPatient {
    /// Unique identifier for the patient
    pub id: PatientId,
    
    /// Reference to patient's profile information
    pub profile_id: ProfileId,
    
    /// External patient code from legacy systems
    pub patient_id: ExternalPatientCode,
    
    /// Patient's blood type with semantic validation
    pub blood_type: Option<BloodType>,
    
    /// Emergency contact name with validation
    pub emergency_contact_name: Option<ContactName>,
    
    /// Emergency contact phone with validation
    pub emergency_contact_phone: Option<PhoneNumber>,
    
    /// Insurance provider name
    pub insurance_provider: Option<InsuranceProvider>,
    
    /// Insurance policy number
    pub insurance_policy_number: Option<PolicyNumber>,
    
    /// Date patient was admitted to hospital
    pub admission_date: Option<AdmissionDate>,
    
    /// Date patient was discharged from hospital
    pub discharge_date: Option<DischargeDate>,
    
    /// Current patient status
    pub status: PatientStatus,
    
    /// When this record was created
    pub created_at: CreatedAt,
    
    /// When this record was last updated
    pub updated_at: UpdatedAt,
}

impl HospitalPatient {
    /// Create a new hospital patient
    pub fn new(
        profile_id: ProfileId,
        patient_id: String,
    ) -> Self {
        Self {
            id: new_patient_id(),
            profile_id,
            patient_id: patient_id.into(),
            blood_type: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            insurance_provider: None,
            insurance_policy_number: None,
            admission_date: None,
            discharge_date: None,
            status: PatientStatus::Outpatient,
            created_at: CreatedAt::now(),
            updated_at: UpdatedAt::now(),
        }
    }
    
    /// Admit the patient to the hospital
    pub fn admit(&mut self) -> Result<(), String> {
        if self.status == PatientStatus::Admitted {
            return Err("Patient is already admitted".to_string());
        }
        
        self.admission_date = Some(AdmissionDate::new(self.updated_at.value()));
        self.discharge_date = None;
        self.status = PatientStatus::Admitted;
        self.updated_at = UpdatedAt::now();
        
        Ok(())
    }
    
    /// Discharge the patient from the hospital
    pub fn discharge(&mut self) -> Result<(), String> {
        if self.status != PatientStatus::Admitted {
            return Err("Patient is not currently admitted".to_string());
        }
        
        self.discharge_date = Some(DischargeDate::new(self.updated_at.value()));
        self.status = PatientStatus::Discharged;
        self.updated_at = UpdatedAt::now();
        
        Ok(())
    }
    
    /// Set emergency contact information
    pub fn set_emergency_contact(
        &mut self,
        name: &str,
        phone: &str,
    ) -> Result<(), String> {
        let contact_name = ContactName::new(name)
            .map_err(|e| e.to_string())?;
        
        let phone_number = PhoneNumber::new(phone)
            .map_err(|e| e.to_string())?;
        
        self.emergency_contact_name = Some(contact_name);
        self.emergency_contact_phone = Some(phone_number);
        self.updated_at = UpdatedAt::now();
        
        Ok(())
    }
    
    /// Set insurance information
    pub fn set_insurance(
        &mut self,
        provider: &str,
        policy_number: &str,
    ) -> Result<(), String> {
        let insurance_provider = InsuranceProvider::new(provider)
            .map_err(|e| e.to_string())?;
        
        let policy = PolicyNumber::new(policy_number)
            .map_err(|e| e.to_string())?;
        
        self.insurance_provider = Some(insurance_provider);
        self.insurance_policy_number = Some(policy);
        self.updated_at = UpdatedAt::now();
        
        Ok(())
    }
    
    /// Get formatted emergency contact information
    pub fn get_emergency_contact_display(&self) -> Option<String> {
        match (&self.emergency_contact_name, &self.emergency_contact_phone) {
            (Some(name), Some(phone)) => {
                Some(format!("{} ({})", name.as_str(), format_phone_number(phone)))
            }
            (Some(name), None) => Some(name.as_string()),
            (None, Some(phone)) => Some(format_phone_number(phone)),
            (None, None) => None,
        }
    }
    
    /// Check if patient is currently admitted
    pub fn is_admitted(&self) -> bool {
        self.status == PatientStatus::Admitted
    }
    
    /// Get length of stay in days (if admitted)
    pub fn length_of_stay_days(&self) -> Option<i64> {
        match (self.admission_date.clone(), self.discharge_date.clone()) {
            (Some(admission), Some(discharge)) => {
                let duration = discharge.value().signed_duration_since(admission.value());
                Some(duration.num_days())
            }
            (Some(admission), None) => {
                let duration = Utc::now().signed_duration_since(admission.value());
                Some(duration.num_days())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_patient_creation() {
        let profile_id = ProfileId::new_v4();
        let patient_code = "P12345".to_string();
        
        let patient = HospitalPatient::new(profile_id, patient_code);
        
        assert_eq!(patient.status, PatientStatus::Outpatient);
        assert!(patient.admission_date.is_none());
        assert!(patient.discharge_date.is_none());
    }
    
    #[test]
    fn test_patient_admission() {
        let mut patient = HospitalPatient::new(
            ProfileId::new_v4(),
            ExternalPatientCode("P12345".to_string().into()),
        );
        
        // Admit patient
        assert!(patient.admit().is_ok());
        assert_eq!(patient.status, PatientStatus::Admitted);
        assert!(patient.admission_date.is_some());
        assert!(patient.discharge_date.is_none());
        
        // Try to admit again
        assert!(patient.admit().is_err());
    }
    
    #[test]
    fn test_patient_discharge() {
        let mut patient = HospitalPatient::new(
            ProfileId::new_v4(),
            ExternalPatientCode("P12345".to_string().into()),
        );
        
        // Try to discharge without admission
        assert!(patient.discharge().is_err());
        
        // Admit then discharge
        assert!(patient.admit().is_ok());
        assert!(patient.discharge().is_ok());
        assert_eq!(patient.status, PatientStatus::Discharged);
        assert!(patient.discharge_date.is_some());
    }
    
    #[test]
    fn test_emergency_contact() {
        let mut patient = HospitalPatient::new(
            ProfileId::new_v4(),
            ExternalPatientCode("P12345".to_string().into()),
        );
        
        // Set valid emergency contact
        assert!(patient.set_emergency_contact("John Doe", "555-123-4567").is_ok());
        assert!(patient.emergency_contact_name.is_some());
        assert!(patient.emergency_contact_phone.is_some());
        
        // Try invalid phone number
        assert!(patient.set_emergency_contact("Jane Doe", "123").is_err());
    }
    
    #[test]
    fn test_insurance() {
        let mut patient = HospitalPatient::new(
            ProfileId::new_v4(),
            ExternalPatientCode("P12345".to_string().into()),
        );
        
        // Set valid insurance
        assert!(patient.set_insurance("Blue Cross", "BC123456789").is_ok());
        assert!(patient.insurance_provider.is_some());
        assert!(patient.insurance_policy_number.is_some());
        
        // Try empty provider
        assert!(patient.set_insurance("", "POL123").is_err());
    }
    
    #[test]
    fn test_length_of_stay() {
        let mut patient = HospitalPatient::new(
            ProfileId::new_v4(),
            ExternalPatientCode("P12345".to_string().into()),
        );
        
        // No stay yet
        assert!(patient.length_of_stay_days().is_none());
        
        // Admit patient
        let admission_time = Utc::now();
        assert!(patient.admit().is_ok());
        
        // Should have some stay duration
        let stay_days = patient.length_of_stay_days();
        assert!(stay_days.is_some());
        assert!(stay_days.unwrap() >= 0);
    }
}
