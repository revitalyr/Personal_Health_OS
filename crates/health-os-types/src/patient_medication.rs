// Refactored Patient Medication using semantic types
// This demonstrates how semantic types improve readability and type safety

use crate::semantic_types::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Patient medication record with semantic type aliases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientMedication {
    /// Unique identifier for the medication record
    pub id: MedicationRecordId,
    
    /// Reference to the patient
    pub patient_id: PatientId,
    
    /// Name of the medication
    pub medication_name: MedicationName,
    
    /// Dosage information
    pub dosage: Dosage,
    
    /// How often to take the medication
    pub frequency: Frequency,
    
    /// Route of administration
    pub route: AdministrationRoute,
    
    /// When medication was started
    pub start_date: StartDate,
    
    /// When medication was ended (optional for ongoing medications)
    pub end_date: Option<EndDate>,
    
    /// Who prescribed the medication
    pub prescribed_by: PrescriberId,
    
    /// Current status of the medication
    pub medication_status: MedicationStatus,
    
    /// When this record was created
    pub created_at: CreatedAt,
}

impl PatientMedication {
    /// Create a new medication record
    pub fn new(
        patient_id: PatientId,
        medication_name: &str,
        dosage: &str,
        frequency: &str,
        route: AdministrationRoute,
        start_date: NaiveDate,
        prescribed_by: PrescriberId,
    ) -> Result<Self, String> {
        let validated_name = MedicationName::new(medication_name)
            .map_err(|e| e.to_string())?;
        
        let validated_dosage = Dosage::new(dosage)
            .map_err(|e| e.to_string())?;
        
        let validated_frequency = Frequency::new(frequency)
            .map_err(|e| e.to_string())?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            patient_id,
            medication_name: validated_name,
            dosage: validated_dosage,
            frequency: validated_frequency,
            route,
            start_date: StartDate::new(start_date),
            end_date: None,
            prescribed_by,
            medication_status: MedicationStatus::Active,
            created_at: CreatedAt::now(),
        })
    }
    
    /// End the medication
    pub fn end_medication(&mut self, end_date: NaiveDate) -> Result<(), String> {
        if self.medication_status == MedicationStatus::Completed {
            return Err("Medication is already completed".to_string());
        }
        
        if self.medication_status == MedicationStatus::Discontinued {
            return Err("Medication is already discontinued".to_string());
        }
        
        // Validate that end date is after start date
        if end_date < self.start_date.value() {
            return Err("End date cannot be before start date".to_string());
        }
        
        self.end_date = Some(EndDate::new(end_date));
        self.medication_status = MedicationStatus::Completed;
        
        Ok(())
    }
    
    /// Discontinue the medication
    pub fn discontinue_medication(&mut self, reason: &str) -> Result<(), String> {
        if self.medication_status == MedicationStatus::Discontinued {
            return Err("Medication is already discontinued".to_string());
        }
        
        if self.medication_status == MedicationStatus::Completed {
            return Err("Cannot discontinue completed medication".to_string());
        }
        
        self.end_date = Some(EndDate::new(chrono::Utc::now().date_naive()));
        self.medication_status = MedicationStatus::Discontinued;
        
        // Store reason in notes if needed (this would require a notes field)
        // For now, we just update the status
        
        Ok(())
    }
    
    /// Put medication on hold
    pub fn put_on_hold(&mut self) -> Result<(), String> {
        if self.medication_status == MedicationStatus::OnHold {
            return Err("Medication is already on hold".to_string());
        }
        
        if self.medication_status != MedicationStatus::Active {
            return Err("Only active medications can be put on hold".to_string());
        }
        
        self.medication_status = MedicationStatus::OnHold;
        Ok(())
    }
    
    /// Resume medication from hold
    pub fn resume_medication(&mut self) -> Result<(), String> {
        if self.medication_status != MedicationStatus::OnHold {
            return Err("Only medications on hold can be resumed".to_string());
        }
        
        self.medication_status = MedicationStatus::Active;
        Ok(())
    }
    
    /// Update dosage information
    pub fn update_dosage(&mut self, new_dosage: &str) -> Result<(), String> {
        let validated_dosage = Dosage::new(new_dosage)
            .map_err(|e| e.to_string())?;
        
        self.dosage = validated_dosage;
        Ok(())
    }
    
    /// Update frequency information
    pub fn update_frequency(&mut self, new_frequency: &str) -> Result<(), String> {
        let validated_frequency = Frequency::new(new_frequency)
            .map_err(|e| e.to_string())?;
        
        self.frequency = validated_frequency;
        Ok(())
    }
    
    /// Check if medication is currently active
    pub fn is_active(&self) -> bool {
        self.medication_status == MedicationStatus::Active
    }
    
    /// Check if medication is completed
    pub fn is_completed(&self) -> bool {
        self.medication_status == MedicationStatus::Completed
    }
    
    /// Check if medication is discontinued
    pub fn is_discontinued(&self) -> bool {
        self.medication_status == MedicationStatus::Discontinued
    }
    
    /// Check if medication is on hold
    pub fn is_on_hold(&self) -> bool {
        self.medication_status == MedicationStatus::OnHold
    }
    
    /// Get duration of medication in days
    pub fn duration_days(&self) -> Option<i64> {
        let end_date = match self.end_date {
            Some(end) => end.value(),
            None => chrono::Utc::now().date_naive(),
        };
        
        let duration = end_date.signed_duration_since(self.start_date.value());
        Some(duration.num_days())
    }
    
    /// Get route display name
    pub fn route_display(&self) -> &'static str {
        match self.route {
            AdministrationRoute::Oral => "Oral",
            AdministrationRoute::Intravenous => "Intravenous",
            AdministrationRoute::Intramuscular => "Intramuscular",
            AdministrationRoute::Subcutaneous => "Subcutaneous",
            AdministrationRoute::Topical => "Topical",
            AdministrationRoute::Inhalation => "Inhalation",
        }
    }
    
    /// Get status display name
    pub fn status_display(&self) -> &'static str {
        match self.medication_status {
            MedicationStatus::Active => "Active",
            MedicationStatus::Completed => "Completed",
            MedicationStatus::Discontinued => "Discontinued",
            MedicationStatus::OnHold => "On Hold",
        }
    }
    
    /// Check if medication requires special monitoring
    pub fn requires_monitoring(&self) -> bool {
        matches!(self.route, 
            AdministrationRoute::Intravenous | 
            AdministrationRoute::Intramuscular |
            AdministrationRoute::Subcutaneous
        )
    }
    
    /// Create a comprehensive medication summary
    pub fn create_summary(&self) -> MedicationSummary {
        MedicationSummary {
            medication_name: self.medication_name.as_string(),
            dosage: self.dosage.as_string(),
            frequency: self.frequency.as_string(),
            route: self.route_display().to_string(),
            status: self.status_display().to_string(),
            start_date: self.start_date.value(),
            end_date: self.end_date.map(|d| d.value()),
            duration_days: self.duration_days(),
            requires_monitoring: self.requires_monitoring(),
            is_active: self.is_active(),
        }
    }
}

/// Comprehensive medication summary for display and reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationSummary {
    pub medication_name: String,
    pub dosage: String,
    pub frequency: String,
    pub route: String,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub duration_days: Option<i64>,
    pub requires_monitoring: bool,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_medication_creation() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        let medication = PatientMedication::new(
            patient_id,
            "Lisinopril",
            "10mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        );
        
        assert!(medication.is_ok());
        let med = medication.unwrap();
        assert_eq!(med.medication_name.as_str(), "Lisinopril");
        assert_eq!(med.dosage.as_str(), "10mg");
        assert_eq!(med.frequency.as_str(), "Once daily");
        assert_eq!(med.route, AdministrationRoute::Oral);
        assert!(med.is_active());
    }
    
    #[test]
    fn test_medication_creation_invalid() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        // Empty medication name
        let medication = PatientMedication::new(
            patient_id,
            "",
            "10mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        );
        assert!(medication.is_err());
        
        // Empty dosage
        let medication = PatientMedication::new(
            patient_id,
            "Lisinopril",
            "",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        );
        assert!(medication.is_err());
    }
    
    #[test]
    fn test_medication_completion() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Amoxicillin",
            "500mg",
            "Three times daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        
        // Complete medication
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 22).unwrap();
        assert!(medication.end_medication(end_date).is_ok());
        assert!(medication.is_completed());
        assert!(!medication.is_active());
        assert_eq!(medication.duration_days(), Some(7));
        
        // Try to complete again
        assert!(medication.end_medication(end_date).is_err());
    }
    
    #[test]
    fn test_medication_discontinuation() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Metformin",
            "500mg",
            "Twice daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        
        // Discontinue medication
        assert!(medication.discontinue_medication("Side effects").is_ok());
        assert!(medication.is_discontinued());
        assert!(!medication.is_active());
        
        // Try to discontinue again
        assert!(medication.discontinue_medication("Side effects").is_err());
    }
    
    #[test]
    fn test_medication_hold_resume() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Warfarin",
            "5mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        
        // Put on hold
        assert!(medication.put_on_hold().is_ok());
        assert!(medication.is_on_hold());
        assert!(!medication.is_active());
        
        // Resume from hold
        assert!(medication.resume_medication().is_ok());
        assert!(medication.is_active());
        assert!(!medication.is_on_hold());
        
        // Try to put on hold again
        assert!(medication.put_on_hold().is_ok());
        assert!(medication.is_on_hold());
        
        // Try to resume when not on hold
        assert!(medication.resume_medication().is_ok());
        assert!(medication.is_active());
    }
    
    #[test]
    fn test_route_display() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        let oral = PatientMedication::new(
            patient_id,
            "Aspirin",
            "81mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(oral.route_display(), "Oral");
        
        let iv = PatientMedication::new(
            patient_id,
            "Morphine",
            "2mg",
            "As needed",
            AdministrationRoute::Intravenous,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(iv.route_display(), "Intravenous");
        
        let topical = PatientMedication::new(
            patient_id,
            "Hydrocortisone",
            "1%",
            "Twice daily",
            AdministrationRoute::Topical,
            start_date,
            prescriber_id,
        ).unwrap();
        assert_eq!(topical.route_display(), "Topical");
    }
    
    #[test]
    fn test_monitoring_requirements() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        let oral = PatientMedication::new(
            patient_id,
            "Lisinopril",
            "10mg",
            "Once daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        assert!(!oral.requires_monitoring());
        
        let iv = PatientMedication::new(
            patient_id,
            "Heparin",
            "5000 units",
            "Continuous",
            AdministrationRoute::Intravenous,
            start_date,
            prescriber_id,
        ).unwrap();
        assert!(iv.requires_monitoring());
        
        let im = PatientMedication::new(
            patient_id,
            "Vitamin B12",
            "1000mcg",
            "Monthly",
            AdministrationRoute::Intramuscular,
            start_date,
            prescriber_id,
        ).unwrap();
        assert!(im.requires_monitoring());
    }
    
    #[test]
    fn test_medication_summary() {
        let patient_id = PatientId::new_v4();
        let prescriber_id = PrescriberId::new_v4();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut medication = PatientMedication::new(
            patient_id,
            "Metformin",
            "500mg",
            "Twice daily",
            AdministrationRoute::Oral,
            start_date,
            prescriber_id,
        ).unwrap();
        
        let summary = medication.create_summary();
        
        assert_eq!(summary.medication_name, "Metformin");
        assert_eq!(summary.dosage, "500mg");
        assert_eq!(summary.frequency, "Twice daily");
        assert_eq!(summary.route, "Oral");
        assert_eq!(summary.status, "Active");
        assert!(summary.is_active);
        assert!(!summary.requires_monitoring);
    }
}
