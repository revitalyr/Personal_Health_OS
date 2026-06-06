use thiserror::Error;
use crate::types::{PatientId, ExternalPatientCode};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, Error)]
pub enum PatientError {
    #[error("Patient not found: {0}")]
    PatientNotFound(PatientId),
    
    #[error("Patient ID already exists: {0}")]
    PatientIdExists(ExternalPatientCode),
    
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
}

impl IntoResponse for PatientError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            Self::PatientNotFound(_) => (StatusCode::NOT_FOUND, "patient_not_found", self.to_string()),
            Self::PatientIdExists(_) => (StatusCode::CONFLICT, "patient_id_exists", self.to_string()),
            Self::InvalidData { .. } => (StatusCode::BAD_REQUEST, "invalid_data", self.to_string()),
            Self::PatientAlreadyAdmitted(_) => (StatusCode::CONFLICT, "patient_already_admitted", self.to_string()),
            Self::PatientNotAdmitted(_) => (StatusCode::CONFLICT, "patient_not_admitted", self.to_string()),
            Self::EncounterNotFound => (StatusCode::NOT_FOUND, "encounter_not_found", self.to_string()),
            Self::InvalidEncounterDates { .. } => (StatusCode::BAD_REQUEST, "invalid_encounter_dates", self.to_string()),
            Self::AllergyExists(_) => (StatusCode::CONFLICT, "allergy_exists", self.to_string()),
            Self::MedicationExists(_) => (StatusCode::CONFLICT, "medication_exists", self.to_string()),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error", self.to_string()),
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error", self.to_string()),
            Self::Unauthorized { .. } => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            Self::InsufficientPermissions { .. } => (StatusCode::FORBIDDEN, "insufficient_permissions", self.to_string()),
            Self::ServiceUnavailable { .. } => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", self.to_string()),
            Self::ExternalService { .. } => (StatusCode::BAD_GATEWAY, "external_service_error", self.to_string()),
            Self::Configuration(_) => (StatusCode::INTERNAL_SERVER_ERROR, "configuration_error", self.to_string()),
            Self::RateLimitExceeded(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded", self.to_string()),
        };

        let body = json!({
            "error": error_type,
            "message": message,
            "is_client_error": self.is_client_error(),
            "is_server_error": self.is_server_error(),
            "is_retryable": self.is_retryable(),
        });

        (status, Json(body)).into_response()
    }
}

impl PatientError {
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

