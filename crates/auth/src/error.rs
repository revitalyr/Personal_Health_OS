use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Email already exists")]
    EmailAlreadyExists,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Invalid OAuth token")]
    InvalidOAuthToken,
    
    #[error("Invalid OTP code")]
    InvalidOTP,
    
    #[error("OTP code expired")]
    OTPExpired,
    
    #[error("OTP attempts exceeded")]
    OTPAttemptsExceeded,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),
    
    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    
    #[error("UUID parsing error: {0}")]
    UUID(#[from] uuid::Error),
    
    #[error("HTTP client error: {0}")]
    HTTP(#[from] reqwest::Error),
    
    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
    
    #[error("Invalid phone number")]
    InvalidPhoneNumber,
    
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    
    #[error("JWT secret must be at least 32 bytes for HMAC-SHA256 security")]
    InvalidJwtSecretLength,
}
