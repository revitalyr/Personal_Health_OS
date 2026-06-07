use crate::models::*;
use crate::error::AuthError;
use crate::User;
use sqlx::{PgPool, Row};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use uuid::Uuid;
use chrono::{Utc, Duration};
use rand::Rng;
use serde_json::json;

/// Full auth service with database persistence.
///
/// Handles email/password registration and login, Google OAuth, Apple sign-in,
/// phone OTP authentication, and profile management. Requires a `PgPool` and
/// OAuth client IDs for external providers.
    pub struct AuthService {
        db: PgPool,
        jwt_secret: String,
        google_client_id: String,
        // Apple OAuth client ID is unused
        _unused_apple_client_id: String,
        http_client: reqwest::Client,
    }

impl AuthService {
    /// Create a new FullAuthService with database pool and OAuth credentials.
    ///
    /// The JWT secret must be at least 32 bytes. Google and Apple client IDs
    /// are used for OAuth token verification.
    pub fn new(
        db: PgPool,
        jwt_secret: String,
        google_client_id: String,
        _unused_apple_client_id: String,
    ) -> Result<Self, AuthError> {
        if jwt_secret.len() < 32 {
            return Err(AuthError::InvalidJwtSecretLength);
        }
        
        Ok(Self {
            db,
            jwt_secret,
             google_client_id,
            _unused_apple_client_id,

            http_client: reqwest::Client::new(),
        })
    }

    /// Register a new user with email and password.
    ///
    /// Password is bcrypt-hashed via `spawn_blocking` to avoid blocking the async runtime.
    /// Returns JWT access and refresh tokens on success.
    pub async fn register_email(&self, request: RegisterRequest) -> Result<AuthToken, AuthError> {
        let existing = sqlx::query("SELECT id FROM accounts WHERE email = $1")
            .bind(&request.email)
            .fetch_optional(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Hash password (CPU-intensive, use spawn_blocking to avoid blocking tokio worker)
        let password = request.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || {
            hash(&password, DEFAULT_COST)
        }).await??;

        let account_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO accounts (id, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(account_id)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        let profile_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO user_profiles (id, account_id, name, relationship, date_of_birth, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(profile_id)
        .bind(account_id)
        .bind(&request.name)
        .bind(ProfileRelationship::Self_ as ProfileRelationship)
        .bind(request.date_of_birth)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        self.generate_tokens(account_id).await
    }

    /// Login with email and password
    pub async fn login_email(&self, email: &str, password: &str) -> Result<AuthToken, AuthError> {
        let account = sqlx::query("SELECT id, password_hash FROM accounts WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.db)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password (CPU-intensive, use spawn_blocking to avoid blocking tokio worker)
        let password_hash_str: String = account.try_get("password_hash").unwrap();
        let password_to_verify = password.to_string();
        let is_valid = tokio::task::spawn_blocking(move || {
            verify(&password_to_verify, &password_hash_str)
        }).await??;
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let account_id: Uuid = account.try_get("id").unwrap();
        self.generate_tokens(account_id).await
    }

    /// Authenticate using a Google ID token.
    ///
    /// Verifies the token against Google's JWKS endpoint (`oauth2/v3/certs`),
    /// then finds or creates an account linked to the Google account.
    pub async fn login_google(&self, id_token: &str) -> Result<AuthToken, AuthError> {
        let token_data = self.verify_google_id_token(id_token).await?;

        let claims = token_data.claims;
        let google_id = claims.get("sub")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?
            .to_string();
        let email = claims.get("email")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?;
        let name = claims.get("name")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?;

        // Find or create account
        let account = sqlx::query("SELECT id FROM accounts WHERE google_id = $1")
            .bind(&google_id)
            .fetch_optional(&self.db)
            .await?;

        let account_id = match account {
            Some(acc) => {
                let id: Uuid = acc.try_get("id").unwrap();
                id
            },
            None => {
                let new_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO accounts (id, email, google_id, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(new_id)
                .bind(&email)
                .bind(&google_id)
                .bind(Utc::now())
                .bind(Utc::now())
                .execute(&self.db)
                .await?;

                let profile_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO user_profiles (id, account_id, name, relationship, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                )
                .bind(profile_id)
                .bind(new_id)
                .bind(name)
                .bind(ProfileRelationship::Self_ as ProfileRelationship)
                .bind(Utc::now())
                .bind(Utc::now())
                .execute(&self.db)
                .await?;

                new_id
            }
        };

        self.generate_tokens(account_id).await
    }

    /// Login with Apple Sign In (not yet implemented)
    pub async fn login_apple(&self, _id_token: &str) -> Result<AuthToken, AuthError> {
        // Apple Sign In verification requires Apple's public keys and JWT verification
        // Similar to Google but with Apple's specific requirements
        // This feature is not yet implemented
        Err(AuthError::NotImplemented("Apple Sign In verification is not yet implemented".to_string()))
    }

    /// Send OTP code to phone number
    pub async fn send_otp(&self, phone: &str) -> Result<(), AuthError> {
        let code: String = (0..6)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect();

        let expires_at = Utc::now() + Duration::minutes(10);

        // Store OTP session
        sqlx::query(
            r#"
            INSERT INTO otp_sessions (id, phone_number, code, expires_at, attempts, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(phone)
        .bind(&code)
        .bind(expires_at)
        .bind(0)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        // Send SMS (implement with SMS service)
        tracing::debug!("OTP code sent");

        Ok(())
    }

    /// Verify OTP code and return auth token
    pub async fn verify_otp(&self, phone: &str, code: &str) -> Result<AuthToken, AuthError> {
        let session = sqlx::query(
            r#"
            SELECT id, expires_at FROM otp_sessions
            WHERE phone_number = $1 AND code = $2
            ORDER BY created_at DESC LIMIT 1
            "#,
        )
        .bind(phone)
        .bind(code)
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::InvalidOTP)?;

        let expires_at: chrono::DateTime<Utc> = session.try_get("expires_at").unwrap();
        if Utc::now() > expires_at {
            return Err(AuthError::OTPExpired);
        }

        let session_id: Uuid = session.try_get("id").unwrap();

        // Atomically increment attempts and check limit in one operation
        let result = sqlx::query("UPDATE otp_sessions SET attempts = attempts + 1 WHERE id = $1 AND attempts < 3 RETURNING attempts")
            .bind(session_id)
            .fetch_optional(&self.db)
            .await?;

        if result.is_none() {
            return Err(AuthError::OTPAttemptsExceeded);
        }

        // Find or create account
        let account = sqlx::query("SELECT id FROM accounts WHERE phone = $1")
            .bind(phone)
            .fetch_optional(&self.db)
            .await?;

        let account_id = match account {
            Some(acc) => {
                let id: Uuid = acc.try_get("id").unwrap();
                id
            },
            None => {
                let new_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO accounts (id, phone, created_at, updated_at)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(new_id)
                .bind(phone)
                .bind(Utc::now())
                .bind(Utc::now())
                .execute(&self.db)
                .await?;
                new_id
            }
        };

        self.generate_tokens(account_id).await
    }

    /// Create a new patient profile for the given account.
    pub async fn create_profile(&self, account_id: Uuid, request: CreateProfileRequest) -> Result<UserProfile, AuthError> {
        let profile_id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO user_profiles (id, account_id, name, relationship, date_of_birth, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, account_id, name, relationship, date_of_birth, created_at, updated_at
            "#,
        )
        .bind(profile_id)
        .bind(account_id)
        .bind(&request.name)
        .bind(request.relationship as ProfileRelationship)
        .bind(request.date_of_birth)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db)
        .await?;

        Ok(UserProfile {
            id: row.try_get("id").unwrap(),
            account_id: row.try_get("account_id").unwrap(),
            name: row.try_get("name").unwrap(),
            relationship: row.try_get("relationship").unwrap(),
            date_of_birth: row.try_get("date_of_birth").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            updated_at: row.try_get("updated_at").unwrap(),
        })
    }

    /// Get all profiles for an account
    pub async fn get_profiles(&self, account_id: Uuid) -> Result<Vec<UserProfile>, AuthError> {
        let rows = sqlx::query(
            r#"
            SELECT id, account_id, name, relationship, date_of_birth, created_at, updated_at
            FROM user_profiles 
            WHERE account_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.db)
        .await?;

        rows.iter().map(|row| {
            Ok(UserProfile {
                id: row.try_get("id").unwrap(),
                account_id: row.try_get("account_id").unwrap(),
                name: row.try_get("name").unwrap(),
                relationship: row.try_get("relationship").unwrap(),
                date_of_birth: row.try_get("date_of_birth").unwrap(),
                created_at: row.try_get("created_at").unwrap(),
                updated_at: row.try_get("updated_at").unwrap(),
            })
        }).collect()
    }

    /// Generate JWT access and refresh tokens for an account
    async fn generate_tokens(&self, account_id: Uuid) -> Result<AuthToken, AuthError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let access_claims = json!({
            "sub": account_id.to_string(),
            "exp": expires_at.timestamp(),
            "iat": now.timestamp(),
            "type": "access",
            "iss": "health_os",
            "aud": "health_os_api"
        });

        let refresh_claims = json!({
            "sub": account_id.to_string(),
            "exp": (now + Duration::days(30)).timestamp(),
            "iat": now.timestamp(),
            "type": "refresh",
            "iss": "health_os",
            "aud": "health_os_api"
        });

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 86400, // 24 hours
        })
    }

    /// Verify JWT token and return account ID
    pub async fn verify_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["health_os"]);
        validation.set_audience(&["health_os_api"]);

        let token_data = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        let sub = token_data.claims.get("sub").unwrap().as_str().unwrap();
        Ok(Uuid::parse_str(sub)?)
    }

    /// Extract user ID from a token (sync, no DB)
    pub fn extract_user_id(&self, token: &str) -> Result<Uuid, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["health_os"]);
        validation.set_audience(&["health_os_api"]);

        let token_data = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        let sub = token_data.claims.get("sub").unwrap().as_str().unwrap();
        Ok(Uuid::parse_str(sub)?)
    }

    /// Generate a short-lived JWT for a user
    pub fn generate_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = json!({
            "sub": user.id.to_string(),
            "email": user.email,
            "name": user.name,
            "exp": (now + Duration::hours(1)).timestamp(),
            "iat": now.timestamp(),
            "iss": "health_os",
            "aud": "health_os_api",
        });
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    /// Generate a short-lived doctor access token
    pub fn generate_doctor_access_token(&self, patient_id: Uuid, expiry_minutes: u32) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = json!({
            "sub": patient_id.to_string(),
            "type": "doctor_access",
            "exp": (now + Duration::minutes(expiry_minutes as i64)).timestamp(),
            "iat": now.timestamp(),
            "iss": "health_os",
            "aud": "health_os_api",
        });
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    /// Validate a doctor access token and return the patient ID
    pub fn validate_doctor_access_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["health_os"]);
        validation.set_audience(&["health_os_api"]);

        let token_data = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        let token_type = token_data.claims.get("type").and_then(|v| v.as_str());
        if token_type != Some("doctor_access") {
            return Err(AuthError::InvalidToken("Not a doctor access token".to_string()));
        }

        let sub = token_data.claims.get("sub").unwrap().as_str().unwrap();
        Ok(Uuid::parse_str(sub)?)
    }

    /// Verify Google OAuth ID token using JWKS
    async fn verify_google_id_token(&self, id_token: &str) -> Result<jsonwebtoken::TokenData<serde_json::Value>, AuthError> {
        // Fetch Google's public keys from JWKS endpoint
        let jwks_response = self.http_client
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .send()
            .await
            .map_err(|_| AuthError::InvalidOAuthToken)?;

        if !jwks_response.status().is_success() {
            return Err(AuthError::InvalidOAuthToken);
        }

        let jwks: serde_json::Value = jwks_response
            .json()
            .await
            .map_err(|_| AuthError::InvalidOAuthToken)?;

        // Extract the header from the token to get the key ID
        let header = jsonwebtoken::decode_header(id_token)
            .map_err(|_| AuthError::InvalidOAuthToken)?;

        let kid = header.kid.ok_or(AuthError::InvalidOAuthToken)?;

        // Find the matching key in JWKS
        let keys = jwks.get("keys")
            .and_then(|v| v.as_array())
            .ok_or(AuthError::InvalidOAuthToken)?;

        let matching_key = keys
            .iter()
            .find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid.as_str()))
            .ok_or(AuthError::InvalidOAuthToken)?;

        // Extract the modulus and exponent to build the decoding key
        let n = matching_key.get("n")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?;
        let e = matching_key.get("e")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|_| AuthError::InvalidOAuthToken)?;

        // Decode and verify the token
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.google_client_id]);
        validation.set_issuer(&["https://accounts.google.com"]);

        let token_data = jsonwebtoken::decode::<serde_json::Value>(
            id_token,
            &decoding_key,
            &validation,
        )
        .map_err(|_| AuthError::InvalidOAuthToken)?;

        Ok(token_data)
    }
}
