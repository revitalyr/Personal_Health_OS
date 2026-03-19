use crate::models::*;
use crate::error::AuthError;
use sqlx::PgPool;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use rand::Rng;
use serde_json::json;

pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
    google_client_id: String,
    apple_client_id: String,
}

impl AuthService {
    pub fn new(
        db: PgPool,
        jwt_secret: String,
        google_client_id: String,
        apple_client_id: String,
    ) -> Self {
        Self {
            db,
            jwt_secret,
            google_client_id,
            apple_client_id,
        }
    }

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

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)?;

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

    pub async fn login_email(&self, email: &str, password: &str) -> Result<AuthToken, AuthError> {
        let account = sqlx::query!(
            "SELECT id, password_hash FROM accounts WHERE email = $1",
            email
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        let is_valid = verify(password, &account.password_hash)?;
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        self.generate_tokens(account.id).await
    }

    pub async fn login_google(&self, id_token: &str) -> Result<AuthToken, AuthError> {
        // Verify Google token
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", id_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::InvalidOAuthToken);
        }

        let user_info: serde_json::Value = response.json().await?;
        let google_id = user_info.get("id").unwrap().as_str().unwrap();
        let email = user_info.get("email").unwrap().as_str().unwrap();
        let name = user_info.get("name").unwrap().as_str().unwrap();

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

    pub async fn login_apple(&self, id_token: &str) -> Result<AuthToken, AuthError> {
        // Similar to Google but with Apple's verification
        // For brevity, simplified implementation
        todo!("Implement Apple Sign In verification")
    }

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
        tracing::info!("OTP code for {}: {}", phone, code);

        Ok(())
    }

    pub async fn verify_otp(&self, phone: &str, code: &str) -> Result<AuthToken, AuthError> {
        let session = sqlx::query!(
            r#"
            SELECT id, expires_at, attempts FROM otp_sessions 
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

        if session.attempts >= 3 {
            return Err(AuthError::OTPAttemptsExceeded);
        }

        // Update attempts
        sqlx::query!(
            "UPDATE otp_sessions SET attempts = attempts + 1 WHERE id = $1",
            session.id
        )
        .execute(&self.db)
        .await?;

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

    async fn generate_tokens(&self, account_id: Uuid) -> Result<AuthToken, AuthError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let access_claims = json!({
            "sub": account_id.to_string(),
            "exp": expires_at.timestamp(),
            "iat": now.timestamp(),
            "type": "access"
        });

        let refresh_claims = json!({
            "sub": account_id.to_string(),
            "exp": (now + Duration::days(30)).timestamp(),
            "iat": now.timestamp(),
            "type": "refresh"
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

    pub async fn verify_token(&self, token: &str) -> Result<Uuid, AuthError> {
        let token_data = decode::<serde_json::Value>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        let sub = token_data.claims.get("sub").unwrap().as_str().unwrap();
        Ok(Uuid::parse_str(sub)?)
    }
}
