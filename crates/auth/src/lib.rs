pub mod models;
pub mod service;
pub mod error;

pub use models::*;
pub use service::AuthService as FullAuthService;

pub use error::{AuthError, Result};

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Lightweight JWT-only auth service (no database required).
///
/// Use this service in microservices that only need to verify tokens
/// or generate short-lived access tokens. For full auth operations
/// (register, login, OTP, profiles), use [`FullAuthService`].
#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
    iss: String,
    aud: String,
}

impl AuthService {
    /// Create a new AuthService with the given JWT signing secret.
    ///
    /// The secret must be at least 32 bytes long (HMAC-SHA256 minimum).
    pub fn new(jwt_secret: &str) -> Result<Self> {
        if jwt_secret.len() < 32 {
            return Err(AuthError::InvalidJwtSecretLength);
        }
        Ok(Self {
            jwt_secret: jwt_secret.to_string(),
            iss: "health_os".to_string(),
            aud: "health_os_api".to_string(),
        })
    }

    /// Verify a JWT token and return the user ID from the `sub` claim.
    pub fn verify_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.iss]);
        validation.set_audience(&[&self.aud]);
        let token_data = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;
        let sub = token_data.claims.get("sub").and_then(|v| v.as_str()).ok_or(AuthError::InvalidClaims)?;
        Ok(Uuid::parse_str(sub)?)
    }

    /// Extract the user ID from a JWT token (alias for `verify_token`).
    pub fn extract_user_id(&self, token: &str) -> Result<Uuid> {
        self.verify_token(token)
    }

    /// Generate a 1-hour JWT token for the given user.
    pub fn generate_token(&self, user: &User) -> Result<String> {
        let now = Utc::now();
        let claims = json!({
            "sub": user.id.to_string(),
            "email": user.email,
            "name": user.name,
            "exp": (now + Duration::hours(1)).timestamp(),
            "iat": now.timestamp(),
            "iss": self.iss,
            "aud": self.aud,
        });
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    /// Generate a short-lived JWT token for temporary doctor access.
    ///
    /// The token carries a `type: "doctor_access"` claim for validation.
    /// Default expiry is 15 minutes.
    pub fn generate_doctor_access_token(&self, patient_id: Uuid, expiry_minutes: u32) -> Result<String> {
        let now = Utc::now();
        let claims = json!({
            "sub": patient_id.to_string(),
            "type": "doctor_access",
            "exp": (now + Duration::minutes(expiry_minutes as i64)).timestamp(),
            "iat": now.timestamp(),
            "iss": self.iss,
            "aud": self.aud,
        });
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    /// Validate a doctor access token and return the associated patient ID.
    ///
    /// Rejects tokens without the `type: "doctor_access"` claim.
    pub fn validate_doctor_access_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.iss]);
        validation.set_audience(&[&self.aud]);
        let token_data = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;
        let token_type = token_data.claims.get("type").and_then(|v| v.as_str());
        if token_type != Some("doctor_access") {
            return Err(AuthError::InvalidToken("Not a doctor access token".to_string()));
        }
        let sub = token_data.claims.get("sub").and_then(|v| v.as_str()).ok_or(AuthError::InvalidClaims)?;
        Ok(Uuid::parse_str(sub)?)
    }
}

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
    pub account_id: Uuid,
    /// Profile name
    pub name: String,
    /// Relationship to the account holder (self, child, parent, spouse, etc.)
    pub relationship: String,
    /// Date of birth
    pub date_of_birth: Option<chrono::NaiveDate>,
    /// Profile creation timestamp
    pub created_at: chrono::DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<Utc>,
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
    fn test_user_creation() {
        let user = create_test_user();
        assert_eq!(user.email, "test@example.com");
    }
}
