use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub relationship: ProfileRelationship,
    pub date_of_birth: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ProfileRelationship {
    Self_,
    Child,
    Parent,
    Spouse,
    Sibling,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub google_id: Option<String>,
    pub apple_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProvider {
    pub provider_type: ProviderType,
    pub provider_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Email,
    Google,
    Apple,
    Phone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub country_code: String,
    pub number: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OTPSession {
    pub id: Uuid,
    pub phone_number: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub provider: ProviderType,
    pub token: Option<String>, // For OAuth providers
    pub email: Option<String>,  // For email login
    pub password: Option<String>, // For email login
    pub phone: Option<String>,   // For phone login
    pub otp_code: Option<String>, // For phone verification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub date_of_birth: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub relationship: ProfileRelationship,
    pub date_of_birth: Option<DateTime<Utc>>,
}
