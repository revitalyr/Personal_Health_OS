use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
    pub database_url: String,
    pub timeline_service_url: String,
    pub document_service_url: String,
    pub ai_service_url: String,
    pub doctor_access_url: String,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;

        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/health_os".to_string());

        let timeline_service_url = env::var("TIMELINE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        let document_service_url = env::var("DOCUMENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());

        let ai_service_url = env::var("AI_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8083".to_string());

        let doctor_access_url = env::var("DOCTOR_ACCESS_URL")
            .unwrap_or_else(|_| "http://localhost:8084".to_string());

        let cors_origins: Vec<String> = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".to_string())
            .split(',')
            .map(|s| s.to_string())
            .collect();

        Ok(Config {
            port,
            jwt_secret,
            database_url,
            timeline_service_url,
            document_service_url,
            ai_service_url,
            doctor_access_url,
            cors_origins,
        })
    }
}
