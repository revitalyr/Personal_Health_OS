// Refactored Medical Encounter using semantic types
// This demonstrates how semantic types improve readability and type safety

use crate::semantic_types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Medical encounter with semantic type aliases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEncounter {
    /// Unique identifier for the encounter
    pub id: EncounterId,
    
    /// Reference to the patient
    pub patient_id: PatientId,
    
    /// Reference to the doctor/staff (optional)
    pub doctor_id: Option<DoctorId>,
    
    /// Type of medical encounter
    pub encounter_type: EncounterType,
    
    /// When the encounter started
    pub start_time: EncounterStart,
    
    /// When the encounter ended (optional for ongoing encounters)
    pub end_time: Option<EncounterEnd>,
    
    /// Medical diagnosis (optional)
    pub diagnosis: Option<Diagnosis>,
    
    /// Treatment provided (optional)
    pub treatment: Option<Treatment>,
    
    /// Additional notes about the encounter
    pub notes: Option<Notes>,
    
    /// When this record was created
    pub created_at: CreatedAt,
}

impl MedicalEncounter {
    /// Create a new medical encounter
    pub fn new(
        patient_id: PatientId,
        encounter_type: EncounterType,
    ) -> Self {
        Self {
            id: new_encounter_id(),
            patient_id,
            doctor_id: None,
            encounter_type,
            start_time: EncounterStart::new(Utc::now()),
            end_time: None,
            diagnosis: None,
            treatment: None,
            notes: None,
            created_at: CreatedAt::now(),
        }
    }
    
    /// Assign a doctor to the encounter
    pub fn assign_doctor(&mut self, doctor_id: DoctorId) {
        self.doctor_id = Some(doctor_id);
    }
    
    /// End the encounter
    pub fn end_encounter(&mut self) -> Result<(), String> {
        if self.end_time.is_some() {
            return Err("Encounter has already ended".to_string());
        }
        
        self.end_time = Some(EncounterEnd::new(Utc::now()));
        Ok(())
    }
    
    /// Set diagnosis for the encounter
    pub fn set_diagnosis(&mut self, diagnosis: &str) -> Result<(), String> {
        let validated_diagnosis = Diagnosis::new(diagnosis)
            .map_err(|e| e.to_string())?;
        
        self.diagnosis = Some(validated_diagnosis);
        Ok(())
    }
    
    /// Set treatment for the encounter
    pub fn set_treatment(&mut self, treatment: &str) -> Result<(), String> {
        let validated_treatment = Treatment::new(treatment)
            .map_err(|e| e.to_string())?;
        
        self.treatment = Some(validated_treatment);
        Ok(())
    }
    
    /// Add notes to the encounter
    pub fn add_notes(&mut self, notes: &str) -> Result<(), String> {
        let validated_notes = Notes::new(notes)
            .map_err(|e| e.to_string())?;
        
        self.notes = Some(validated_notes);
        Ok(())
    }
    
    /// Check if encounter is currently active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }
    
    /// Get duration of the encounter (if ended)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match &self.end_time {
            Some(end) => {
                Some(end.value().signed_duration_since(self.start_time.value()))
            }
            None => None,
        }
    }
    
    /// Get duration in minutes (if ended)
    pub fn duration_minutes(&self) -> Option<i64> {
        self.duration().map(|d| d.num_minutes())
    }
    
    /// Check if encounter has diagnosis
    pub fn has_diagnosis(&self) -> bool {
        self.diagnosis.is_some()
    }
    
    /// Check if encounter has treatment
    pub fn has_treatment(&self) -> bool {
        self.treatment.is_some()
    }
    
    /// Get encounter type display name
    pub fn encounter_type_display(&self) -> &'static str {
        match self.encounter_type {
            EncounterType::Consultation => "Consultation",
            EncounterType::Emergency => "Emergency",
            EncounterType::Surgery => "Surgery",
            EncounterType::FollowUp => "Follow-up",
            EncounterType::Diagnostic => "Diagnostic",
            EncounterType::Therapy => "Therapy",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_encounter_creation() {
        let patient_id = PatientId::new_v4();
        let encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        
        assert_eq!(encounter.encounter_type, EncounterType::Consultation);
        assert!(encounter.is_active());
        assert!(encounter.doctor_id.is_none());
        assert!(encounter.diagnosis.is_none());
        assert!(encounter.treatment.is_none());
    }
    
    #[test]
    fn test_doctor_assignment() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Emergency);
        let doctor_id = DoctorId::new_v4();
        
        encounter.assign_doctor(doctor_id);
        assert_eq!(encounter.doctor_id, Some(doctor_id));
    }
    
    #[test]
    fn test_encounter_ending() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Surgery);
        
        // End encounter
        assert!(encounter.end_encounter().is_ok());
        assert!(!encounter.is_active());
        assert!(encounter.duration().is_some());
        
        // Try to end again
        assert!(encounter.end_encounter().is_err());
    }
    
    #[test]
    fn test_diagnosis_setting() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        
        // Set valid diagnosis
        assert!(encounter.set_diagnosis("Hypertension").is_ok());
        assert!(encounter.has_diagnosis());
        
        // Try empty diagnosis
        assert!(encounter.set_diagnosis("").is_err());
    }
    
    #[test]
    fn test_treatment_setting() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Therapy);
        
        // Set valid treatment
        assert!(encounter.set_treatment("Physical therapy sessions").is_ok());
        assert!(encounter.has_treatment());
        
        // Try too long treatment
        let long_treatment = "A".repeat(3000);
        assert!(encounter.set_treatment(&long_treatment).is_err());
    }
    
    #[test]
    fn test_notes_adding() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Emergency);
        
        // Add valid notes
        assert!(encounter.add_notes("Patient responded well to treatment").is_ok());
        assert!(encounter.notes.is_some());
        
        // Try too long notes
        let long_notes = "A".repeat(6000);
        assert!(encounter.add_notes(&long_notes).is_err());
    }
    
    #[test]
    fn test_duration_calculation() {
        let patient_id = PatientId::new_v4();
        let mut encounter = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        
        // No duration yet
        assert!(encounter.duration().is_none());
        assert!(encounter.duration_minutes().is_none());
        
        // End encounter and check duration
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(encounter.end_encounter().is_ok());
        
        let duration = encounter.duration();
        assert!(duration.is_some());
        assert!(duration.unwrap().num_milliseconds() >= 100);
    }
    
    #[test]
    fn test_encounter_type_display() {
        let patient_id = PatientId::new_v4();
        
        let consultation = MedicalEncounter::new(patient_id, EncounterType::Consultation);
        assert_eq!(consultation.encounter_type_display(), "Consultation");
        
        let emergency = MedicalEncounter::new(patient_id, EncounterType::Emergency);
        assert_eq!(emergency.encounter_type_display(), "Emergency");
        
        let surgery = MedicalEncounter::new(patient_id, EncounterType::Surgery);
        assert_eq!(surgery.encounter_type_display(), "Surgery");
    }
}
