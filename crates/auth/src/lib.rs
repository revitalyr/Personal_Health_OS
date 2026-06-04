pub mod models;
pub mod service;
pub mod error;

pub use models::*;
pub use error::*;
// Note: service::AuthService is the full implementation with DB support
// Re-export specific items from service as needed
pub use service::{AuthService as FullAuthService, AuthToken, RegisterRequest, LoginRequest, OTPRequest};

// Legacy compatibility - keep simple types for external use
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// JWT claims structure for authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// User ID (subject)
    pub sub: String,
    /// User email
    pub email: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
    /// Issuer identifier
    pub iss: String,
    /// Audience identifier
    pub aud: String,
}

/// User information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,
    /// User email address
    pub email: String,
    /// User display name
    pub name: String,
    /// Account creation timestamp
    pub created_at: chrono::DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<Utc>,
}

/// User profile information for patient profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique profile identifier
    pub id: Uuid,
    /// Associated user ID
    pub user_id: Uuid,
    /// Profile name
    pub name: String,
    /// Relationship to the account holder (self, child, parent, spouse, etc.)
    pub relationship: String,
    /// Date of birth
    pub date_of_birth: Option<chrono::NaiveDate>,
    /// Profile creation timestamp
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid claims")]
    InvalidClaims,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Encoding error: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),
}

pub type Result<T> = std::result::Result<T, AuthError>;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_token_generation_and_validation() {
        let auth_service = AuthService::new(
            "test_secret".to_string(),
            "health_os".to_string(),
            "health_os_api".to_string(),
        );

        let user = create_test_user();
        let token = auth_service.generate_token(&user).unwrap();
        
        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_doctor_access_token() {
        let auth_service = AuthService::new(
            "test_secret".to_string(),
            "health_os".to_string(),
            "health_os_api".to_string(),
        );

        let patient_id = Uuid::new_v4();
        let token = auth_service.generate_doctor_access_token(patient_id, 15).unwrap();
        
        let extracted_patient_id = auth_service.validate_doctor_access_token(&token).unwrap();
        assert_eq!(extracted_patient_id, patient_id);
    }

    #[test]
    fn test_invalid_token() {
        let auth_service = AuthService::new(
            "test_secret".to_string(),
            "health_os".to_string(),
            "health_os_api".to_string(),
        );

        let result = auth_service.validate_token("invalid_token");
        assert!(result.is_err());
    }
}
