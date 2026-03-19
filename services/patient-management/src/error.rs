use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatientError {
    #[error("Patient not found")]
    PatientNotFound,
    
    #[error("Patient ID already exists")]
    PatientIdExists,
    
    #[error("Invalid patient data: {0}")]
    InvalidData(String),
    
    #[error("Cannot admit patient - already admitted")]
    PatientAlreadyAdmitted,
    
    #[error("Cannot discharge patient - not admitted")]
    PatientNotAdmitted,
    
    #[error("Medical encounter not found")]
    EncounterNotFound,
    
    #[error("Invalid encounter dates")]
    InvalidEncounterDates,
    
    #[error("Allergy already exists")]
    AllergyExists,
    
    #[error("Medication already prescribed")]
    MedicationExists,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Service temporarily unavailable")]
    ServiceUnavailable,
}
