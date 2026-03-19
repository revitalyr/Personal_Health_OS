pub mod models;
pub mod service;
pub mod error;

pub use models::*;
pub use service::*;
pub use error::*;

// Legacy compatibility
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
    pub iss: String, // Issuer
    pub aud: String, // Audience
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub relationship: String, // self, child, parent, spouse, etc.
    pub date_of_birth: Option<chrono::NaiveDate>,
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

pub struct AuthService {
    jwt_secret: String,
    issuer: String,
    audience: String,
}

impl AuthService {
    pub fn new(jwt_secret: String, issuer: String, audience: String) -> Self {
        Self {
            jwt_secret,
            issuer,
            audience,
        }
    }

    pub fn generate_token(&self, user: &User) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        tracing::info!("Generated JWT token for user: {}", user.id);
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256)
            .with_issuer(&self.issuer)
            .with_audience(&self.audience);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        let claims = token_data.claims;

        // Check if token is expired
        if claims.exp < Utc::now().timestamp() {
            return Err(AuthError::TokenExpired);
        }

        tracing::debug!("Validated JWT token for user: {}", claims.sub);
        Ok(claims)
    }

    pub fn extract_user_id(&self, token: &str) -> Result<Uuid> {
        let claims = self.validate_token(token)?;
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidClaims)?;
        Ok(user_id)
    }

    pub fn generate_doctor_access_token(&self, patient_id: Uuid, duration_minutes: i64) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(duration_minutes);

        let claims = Claims {
            sub: format!("doctor_access:{}", patient_id),
            email: "doctor@healthos.app".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.issuer.clone(),
            aud: "doctor_viewer".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        tracing::info!(
            "Generated doctor access token for patient: {}, expires in {} minutes",
            patient_id,
            duration_minutes
        );
        Ok(token)
    }

    pub fn validate_doctor_access_token(&self, token: &str) -> Result<Uuid> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256)
            .with_issuer(&self.issuer)
            .with_audience(&["doctor_viewer"]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        let claims = token_data.claims;

        // Check if token is expired
        if claims.exp < Utc::now().timestamp() {
            return Err(AuthError::TokenExpired);
        }

        // Extract patient ID from subject
        let patient_id_str = claims.sub.strip_prefix("doctor_access:")
            .ok_or(AuthError::InvalidClaims)?;
        
        let patient_id = Uuid::parse_str(patient_id_str)
            .map_err(|_| AuthError::InvalidClaims)?;

        tracing::debug!("Validated doctor access token for patient: {}", patient_id);
        Ok(patient_id)
    }
}

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
