use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
    pub database_url: String,
    pub google_client_id: String,
    pub apple_client_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string());

        let google_client_id = env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| "google-client-id".to_string());
        let apple_client_id = env::var("APPLE_CLIENT_ID")
            .unwrap_or_else(|_| "apple-client-id".to_string());

        Ok(Config {
            port,
            jwt_secret,
            database_url,
            google_client_id,
            apple_client_id,
        })
    }
}
