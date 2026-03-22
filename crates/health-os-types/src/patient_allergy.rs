// Refactored Patient Allergy using semantic types
// This demonstrates how semantic types improve readability and type safety

use crate::semantic_types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Patient allergy record with semantic type aliases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientAllergy {
    /// Unique identifier for the allergy record
    pub id: AllergyId,
    
    /// Reference to the patient
    pub patient_id: PatientId,
    
    /// Name of the allergen
    pub allergen: AllergenName,
    
    /// Severity of the allergic reaction
    pub severity: AllergySeverity,
    
    /// Description of the reaction (optional)
    pub reaction: Option<AllergyReaction>,
    
    /// Additional notes about the allergy (optional)
    pub notes: Option<AllergyNotes>,
    
    /// When this allergy record was created
    pub created_at: CreatedAt,
}

impl PatientAllergy {
    /// Create a new allergy record
    pub fn new(
        patient_id: PatientId,
        allergen: &str,
        severity: AllergySeverity,
    ) -> Result<Self, String> {
        let validated_allergen = AllergenName::new(allergen)
            .map_err(|e| e.to_string())?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            patient_id,
            allergen: validated_allergen,
            severity,
            reaction: None,
            notes: None,
            created_at: CreatedAt::now(),
        })
    }
    
    /// Set or update the allergic reaction description
    pub fn set_reaction(&mut self, reaction: &str) -> Result<(), String> {
        let validated_reaction = AllergyReaction::new(reaction)
            .map_err(|e| e.to_string())?;
        
        self.reaction = Some(validated_reaction);
        Ok(())
    }
    
    /// Set or update allergy notes
    pub fn set_notes(&mut self, notes: &str) -> Result<(), String> {
        let validated_notes = AllergyNotes::new(notes)
            .map_err(|e| e.to_string())?;
        
        self.notes = Some(validated_notes);
        Ok(())
    }
    
    /// Update severity level
    pub fn update_severity(&mut self, severity: AllergySeverity) {
        self.severity = severity;
    }
    
    /// Check if this is a severe allergy
    pub fn is_severe(&self) -> bool {
        matches!(self.severity, AllergySeverity::Severe | AllergySeverity::LifeThreatening)
    }
    
    /// Check if this is life-threatening
    pub fn is_life_threatening(&self) -> bool {
        matches!(self.severity, AllergySeverity::LifeThreatening)
    }
    
    /// Get severity display name
    pub fn severity_display(&self) -> &'static str {
        match self.severity {
            AllergySeverity::Mild => "Mild",
            AllergySeverity::Moderate => "Moderate",
            AllergySeverity::Severe => "Severe",
            AllergySeverity::LifeThreatening => "Life-Threatening",
        }
    }
    
    /// Get allergy category based on allergen type
    pub fn get_allergy_category(&self) -> AllergyCategory {
        let allergen_lower = self.allergen.as_str().to_lowercase();
        
        if allergen_lower.contains("penicillin") || allergen_lower.contains("antibiotic") {
            AllergyCategory::Medication
        } else if allergen_lower.contains("pollen") || allergen_lower.contains("dust") {
            AllergyCategory::Environmental
        } else if allergen_lower.contains("peanut") || allergen_lower.contains("shellfish") {
            AllergyCategory::Food
        } else if allergen_lower.contains("latex") {
            AllergyCategory::Material
        } else {
            AllergyCategory::Other
        }
    }
    
    /// Get emergency action recommendations
    pub fn get_emergency_actions(&self) -> Vec<String> {
        match self.severity {
            AllergySeverity::Mild => vec![
                "Monitor for symptoms".to_string(),
                "Administer antihistamine if needed".to_string(),
                "Avoid allergen exposure".to_string()
            ],
            AllergySeverity::Moderate => vec![
                "Immediate medical attention recommended".to_string(),
                "Administer epinephrine if available".to_string(),
                "Call emergency services".to_string()
            ],
            AllergySeverity::Severe => vec![
                "Call emergency services immediately".to_string(),
                "Administer epinephrine if available".to_string(),
                "Prepare for anaphylaxis treatment".to_string(),
                "Monitor airway and breathing".to_string()
            ],
            AllergySeverity::LifeThreatening => vec![
                "EMERGENCY: Call 911 immediately".to_string(),
                "Administer epinephrine immediately".to_string(),
                "Prepare for CPR if needed".to_string(),
                "Advanced life support required".to_string(),
                "Rush to emergency department".to_string()
            ],
        }
    }
    
    /// Create a comprehensive allergy summary
    pub fn create_summary(&self) -> AllergySummary {
        AllergySummary {
            allergen: self.allergen.as_str().to_string(),
            severity: self.severity_display().to_string(),
            category: format!("{:?}", self.get_allergy_category()),
            reaction: self.reaction.as_ref().map(|r| r.as_str().to_string()),
            notes: self.notes.as_ref().map(|n| n.as_str().to_string()),
            emergency_actions: self.get_emergency_actions(),
            is_severe: self.is_severe(),
            is_life_threatening: self.is_life_threatening(),
            created_at: self.created_at.value(),
        }
    }
}

/// Categories of allergies for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllergyCategory {
    Medication,
    Food,
    Environmental,
    Material,
    Other,
}

/// Comprehensive allergy summary for display and reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllergySummary {
    pub allergen: String,
    pub severity: String,
    pub category: String,
    pub reaction: Option<String>,
    pub notes: Option<String>,
    pub emergency_actions: Vec<String>,
    pub is_severe: bool,
    pub is_life_threatening: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allergy_creation() {
        let patient_id = PatientId::new_v4();
        
        let allergy = PatientAllergy::new(
            patient_id,
            "Peanuts",
            AllergySeverity::Severe,
        );
        
        assert!(allergy.is_ok());
        let allergy = allergy.unwrap();
        assert_eq!(allergy.allergen.as_str(), "Peanuts");
        assert_eq!(allergy.severity, AllergySeverity::Severe);
        assert!(allergy.reaction.is_none());
        assert!(allergy.notes.is_none());
    }
    
    #[test]
    fn test_allergy_creation_invalid() {
        let patient_id = PatientId::new_v4();
        
        // Empty allergen name
        let allergy = PatientAllergy::new(
            patient_id,
            "",
            AllergySeverity::Mild,
        );
        assert!(allergy.is_err());
        
        // Too long allergen name
        let long_name = "A".repeat(200);
        let allergy = PatientAllergy::new(
            patient_id,
            &long_name,
            AllergySeverity::Mild,
        );
        assert!(allergy.is_err());
    }
    
    #[test]
    fn test_reaction_setting() {
        let patient_id = PatientId::new_v4();
        let mut allergy = PatientAllergy::new(
            patient_id,
            "Penicillin",
            AllergySeverity::Moderate,
        ).unwrap();
        
        // Set valid reaction
        assert!(allergy.set_reaction("Hives and itching").is_ok());
        assert!(allergy.reaction.is_some());
        
        // Try too long reaction
        let long_reaction = "A".repeat(600);
        assert!(allergy.set_reaction(&long_reaction).is_err());
    }
    
    #[test]
    fn test_notes_setting() {
        let patient_id = PatientId::new_v4();
        let mut allergy = PatientAllergy::new(
            patient_id,
            "Latex",
            AllergySeverity::Mild,
        ).unwrap();
        
        // Set valid notes
        assert!(allergy.set_notes("Mild skin irritation, avoid latex gloves").is_ok());
        assert!(allergy.notes.is_some());
        
        // Try too long notes
        let long_notes = "A".repeat(2000);
        assert!(allergy.set_notes(&long_notes).is_err());
    }
    
    #[test]
    fn test_severity_checks() {
        let patient_id = PatientId::new_v4();
        
        let mild = PatientAllergy::new(
            patient_id,
            "Dust",
            AllergySeverity::Mild,
        ).unwrap();
        assert!(!mild.is_severe());
        assert!(!mild.is_life_threatening());
        
        let severe = PatientAllergy::new(
            patient_id,
            "Bee stings",
            AllergySeverity::Severe,
        ).unwrap();
        assert!(severe.is_severe());
        assert!(!severe.is_life_threatening());
        
        let life_threatening = PatientAllergy::new(
            patient_id,
            "Peanuts",
            AllergySeverity::LifeThreatening,
        ).unwrap();
        assert!(life_threatening.is_severe());
        assert!(life_threatening.is_life_threatening());
    }
    
    #[test]
    fn test_allergy_categories() {
        let patient_id = PatientId::new_v4();
        
        let medication = PatientAllergy::new(
            patient_id,
            "Penicillin",
            AllergySeverity::Moderate,
        ).unwrap();
        assert_eq!(medication.get_allergy_category(), AllergyCategory::Medication);
        
        let food = PatientAllergy::new(
            patient_id,
            "Shellfish",
            AllergySeverity::Severe,
        ).unwrap();
        assert_eq!(food.get_allergy_category(), AllergyCategory::Food);
        
        let environmental = PatientAllergy::new(
            patient_id,
            "Pollen",
            AllergySeverity::Mild,
        ).unwrap();
        assert_eq!(environmental.get_allergy_category(), AllergyCategory::Environmental);
    }
    
    #[test]
    fn test_emergency_actions() {
        let patient_id = PatientId::new_v4();
        
        let mild = PatientAllergy::new(
            patient_id,
            "Dust",
            AllergySeverity::Mild,
        ).unwrap();
        let actions = mild.get_emergency_actions();
        assert!(actions.contains(&"Monitor for symptoms".to_string()));
        assert!(!actions.contains(&"Call emergency services".to_string()));
        
        let life_threatening = PatientAllergy::new(
            patient_id,
            "Peanuts",
            AllergySeverity::LifeThreatening,
        ).unwrap();
        let actions = life_threatening.get_emergency_actions();
        assert!(actions.contains(&"EMERGENCY: Call 911 immediately".to_string()));
        assert!(actions.contains(&"Administer epinephrine immediately".to_string()));
    }
    
    #[test]
    fn test_allergy_summary() {
        let patient_id = PatientId::new_v4();
        let mut allergy = PatientAllergy::new(
            patient_id,
            "Latex",
            AllergySeverity::Moderate,
        ).unwrap();
        
        allergy.set_reaction("Skin rash, itching").unwrap();
        allergy.set_notes("Avoid latex products, use alternative materials").unwrap();
        
        let summary = allergy.create_summary();
        
        assert_eq!(summary.allergen, "Latex");
        assert_eq!(summary.severity, "Moderate");
        assert_eq!(summary.reaction, Some("Skin rash, itching".to_string()));
        assert_eq!(summary.notes, Some("Avoid latex products, use alternative materials".to_string()));
        assert!(summary.emergency_actions.contains(&"Immediate medical attention recommended".to_string()));
    }
}
