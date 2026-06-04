use crate::models::*;
use crate::error::AuthError;
use sqlx::PgPool;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use rand::Rng;
use serde_json::json;

pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
    google_client_id: String,
    apple_client_id: String,
    http_client: reqwest::Client,
}

impl AuthService {
    pub fn new(
        db: PgPool,
        jwt_secret: String,
        google_client_id: String,
        apple_client_id: String,
    ) -> Result<Self, AuthError> {
        // Validate JWT secret minimum length (HMAC-SHA256 requires ≥32 bytes)
        if jwt_secret.len() < 32 {
            return Err(AuthError::InvalidJwtSecretLength);
        }
        
        Ok(Self {
            db,
            jwt_secret,
            google_client_id,
            apple_client_id,
            http_client: reqwest::Client::new(),
        })
    }

    /// Register a new user with email and password
    pub async fn register_email(&self, request: RegisterRequest) -> Result<AuthToken, AuthError> {
        // Check if email already exists
        let existing = sqlx::query!(
            "SELECT id FROM accounts WHERE email = $1",
            request.email
        )
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

        // Create account
        let account_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO accounts (id, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            account_id,
            request.email,
            password_hash,
            Utc::now(),
            Utc::now()
        )
        .execute(&self.db)
        .await?;

        // Create user profile
        let profile_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO user_profiles (id, account_id, name, relationship, date_of_birth, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            profile_id,
            account_id,
            request.name,
            ProfileRelationship::Self_ as ProfileRelationship,
            request.date_of_birth,
            Utc::now(),
            Utc::now()
        )
        .execute(&self.db)
        .await?;

        self.generate_tokens(account_id).await
    }

    /// Login with email and password
    pub async fn login_email(&self, email: &str, password: &str) -> Result<AuthToken, AuthError> {
        let account = sqlx::query!(
            "SELECT id, password_hash FROM accounts WHERE email = $1",
            email
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        // Verify password (CPU-intensive, use spawn_blocking to avoid blocking tokio worker)
        let password_hash = account.password_hash.clone();
        let password_to_verify = password.to_string();
        let is_valid = tokio::task::spawn_blocking(move || {
            verify(&password_to_verify, &password_hash)
        }).await??;
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        self.generate_tokens(account.id).await
    }

    pub async fn login_google(&self, id_token: &str) -> Result<AuthToken, AuthError> {
        // Verify Google ID token signature and claims using JWKS
        let google_id = self.verify_google_id_token(id_token).await?;

        // Decode token to extract claims (email, name)
        let token_data = jsonwebtoken::decode::<serde_json::Value>(
            id_token,
            &DecodingKey::from_secret(b""), // Signature already verified
            &Validation::new(Algorithm::RS256),
        )
        .map_err(|_| AuthError::InvalidOAuthToken)?;

        let claims = token_data.claims;
        let email = claims.get("email")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?;
        let name = claims.get("name")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?;

        // Find or create account
        let account = sqlx::query!(
            "SELECT id FROM accounts WHERE google_id = $1",
            google_id
        )
        .fetch_optional(&self.db)
        .await?;

        let account_id = match account {
            Some(acc) => acc.id,
            None => {
                let new_id = Uuid::new_v4();
                sqlx::query!(
                    r#"
                    INSERT INTO accounts (id, email, google_id, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                    new_id,
                    email,
                    google_id,
                    Utc::now(),
                    Utc::now()
                )
                .execute(&self.db)
                .await?;

                // Create profile
                let profile_id = Uuid::new_v4();
                sqlx::query!(
                    r#"
                    INSERT INTO user_profiles (id, account_id, name, relationship, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    profile_id,
                    new_id,
                    name,
                    ProfileRelationship::Self_ as ProfileRelationship,
                    Utc::now(),
                    Utc::now()
                )
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
        // Generate 6-digit code
        let code: String = (0..6)
            .map(|_| rand::thread_rng().gen_range(0..10).to_string())
            .collect();

        let expires_at = Utc::now() + Duration::minutes(10);

        // Store OTP session
        sqlx::query!(
            r#"
            INSERT INTO otp_sessions (id, phone_number, code, expires_at, attempts, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            Uuid::new_v4(),
            phone,
            code,
            expires_at,
            0,
            Utc::now()
        )
        .execute(&self.db)
        .await?;

        // Send SMS (implement with SMS service)
        tracing::info!("OTP code sent to {}", phone);

        Ok(())
    }

    /// Verify OTP code and return auth token
    pub async fn verify_otp(&self, phone: &str, code: &str) -> Result<AuthToken, AuthError> {
        let session = sqlx::query!(
            r#"
            SELECT id, expires_at FROM otp_sessions
            WHERE phone_number = $1 AND code = $2
            ORDER BY created_at DESC LIMIT 1
            "#,
            phone,
            code
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::InvalidOTP)?;

        if Utc::now() > session.expires_at {
            return Err(AuthError::OTPExpired);
        }

        // Atomically increment attempts and check limit in one operation
        let result = sqlx::query!(
            "UPDATE otp_sessions SET attempts = attempts + 1 WHERE id = $1 AND attempts < 3 RETURNING attempts",
            session.id
        )
        .fetch_optional(&self.db)
        .await?;

        if result.is_none() {
            return Err(AuthError::OTPAttemptsExceeded);
        }

        // Find or create account
        let account = sqlx::query!(
            "SELECT id FROM accounts WHERE phone = $1",
            phone
        )
        .fetch_optional(&self.db)
        .await?;

        let account_id = match account {
            Some(acc) => acc.id,
            None => {
                let new_id = Uuid::new_v4();
                sqlx::query!(
                    r#"
                    INSERT INTO accounts (id, phone, created_at, updated_at)
                    VALUES ($1, $2, $3, $4)
                    "#,
                    new_id,
                    phone,
                    Utc::now(),
                    Utc::now()
                )
                .execute(&self.db)
                .await?;
                new_id
            }
        };

        self.generate_tokens(account_id).await
    }

    pub async fn create_profile(&self, account_id: Uuid, request: CreateProfileRequest) -> Result<UserProfile, AuthError> {
        let profile_id = Uuid::new_v4();
        sqlx::query_as!(
            UserProfile,
            r#"
            INSERT INTO user_profiles (id, account_id, name, relationship, date_of_birth, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, account_id, name, relationship as "relationship: ProfileRelationship", date_of_birth, created_at, updated_at
            "#,
            profile_id,
            account_id,
            request.name,
            request.relationship as ProfileRelationship,
            request.date_of_birth,
            Utc::now(),
            Utc::now()
        )
        .fetch_one(&self.db)
        .await
    }

    /// Get all profiles for an account
    pub async fn get_profiles(&self, account_id: Uuid) -> Result<Vec<UserProfile>, AuthError> {
        sqlx::query_as!(
            UserProfile,
            r#"
            SELECT id, account_id, name, relationship as "relationship: ProfileRelationship", 
                   date_of_birth, created_at, updated_at
            FROM user_profiles 
            WHERE account_id = $1
            ORDER BY created_at
            "#,
            account_id
        )
        .fetch_all(&self.db)
        .await
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

    /// Verify Google OAuth ID token using JWKS
    async fn verify_google_id_token(&self, id_token: &str) -> Result<String, AuthError> {
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

        // Extract the subject (google_id)
        let google_id = token_data.claims
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidOAuthToken)?
            .to_string();

        Ok(google_id)
    }
}
