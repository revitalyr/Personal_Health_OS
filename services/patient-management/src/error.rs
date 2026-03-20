use thiserror::Error;
use crate::models::PatientId;

#[derive(Debug, Error)]
pub enum PatientError {
    #[error("Patient not found: {0}")]
    PatientNotFound(PatientId),
    
    #[error("Patient ID already exists: {0}")]
    PatientIdExists(PatientId),
    
    #[error("Invalid patient data: {field} - {message}")]
    InvalidData { field: String, message: String },
    
    #[error("Cannot admit patient {0} - already admitted")]
    PatientAlreadyAdmitted(PatientId),
    
    #[error("Cannot discharge patient {0} - not admitted")]
    PatientNotAdmitted(PatientId),
    
    #[error("Medical encounter not found")]
    EncounterNotFound,
    
    #[error("Invalid encounter dates: start {start:?} must be before end {end:?}")]
    InvalidEncounterDates { start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc> },
    
    #[error("Allergy already exists for patient {0}")]
    AllergyExists(PatientId),
    
    #[error("Medication already prescribed for patient {0}")]
    MedicationExists(PatientId),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    
    #[error("Unauthorized access: {reason}")]
    Unauthorized { reason: String },
    
    #[error("Insufficient permissions for action {action}")]
    InsufficientPermissions { action: String },
    
    #[error("Service temporarily unavailable: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Rate limit exceeded for {0}")]
    RateLimitExceeded(String),
}

pub type Result<T> = std::result::Result<T, PatientError>;

impl PatientError {
    pub fn invalid_data(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidData {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn unauthorized(reason: impl Into<String>) -> Self {
        Self::Unauthorized {
            reason: reason.into(),
        }
    }
    
    pub fn insufficient_permissions(action: impl Into<String>) -> Self {
        Self::InsufficientPermissions {
            action: action.into(),
        }
    }
    
    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            service: service.into(),
        }
    }
    
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }
    
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }
    
    pub fn rate_limit_exceeded(resource: impl Into<String>) -> Self {
        Self::RateLimitExceeded(resource.into())
    }
    
    /// Returns true if this error is client-side (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::PatientNotFound(_)
                | Self::PatientIdExists(_)
                | Self::InvalidData { .. }
                | Self::PatientAlreadyAdmitted(_)
                | Self::PatientNotAdmitted(_)
                | Self::EncounterNotFound
                | Self::InvalidEncounterDates { .. }
                | Self::AllergyExists(_)
                | Self::MedicationExists(_)
                | Self::Validation(_)
                | Self::Unauthorized { .. }
                | Self::InsufficientPermissions { .. }
                | Self::RateLimitExceeded(_)
        )
    }
    
    /// Returns true if this error is server-side (5xx)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::ServiceUnavailable { .. }
                | Self::ExternalService { .. }
                | Self::Configuration(_)
        )
    }
    
    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::ServiceUnavailable { .. }
                | Self::ExternalService { .. }
                | Self::RateLimitExceeded(_)
        )
    }
}
