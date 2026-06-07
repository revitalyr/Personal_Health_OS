use serde::{Deserialize, Serialize};

/// A medical entity identified in text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEntity {
    /// The matched text
    pub text: String,
    /// Entity type: medication, diagnosis, symptom, lab_value, date
    pub entity_type: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Start position in source text
    pub start_pos: usize,
    /// End position in source text
    pub end_pos: usize,
    /// Normalized/canonical form
    pub normalized_value: Option<String>,
}
